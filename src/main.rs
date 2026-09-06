mod crypto;
mod models;
mod probe;
mod routes;
use axum::{
    body::Body,
    http::{
        header::{self, HeaderValue},
        Request,
    },
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Json, Router,
};
use crypto::{key_source, SecretBox};
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
    SqlitePool,
};
use std::{
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
    time::Duration,
};
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::KeyExtractor, GovernorError, GovernorLayer,
};
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    limit::RequestBodyLimitLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

#[derive(Clone)]
pub struct AppState {
    db: SqlitePool,
    secrets: SecretBox,
    access_token: Arc<str>,
    /// Only used by the in-process test harness; production never permits
    /// private-network probe targets.
    allow_private_endpoints: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    let data_dir_supplied = std::env::var_os("DATA_DIR").is_some();
    let default_data_dir = if Path::new("/data").is_dir() {
        "/data"
    } else {
        "./data"
    };
    let data_dir = PathBuf::from(
        std::env::var("DATA_DIR").unwrap_or_else(|_| default_data_dir.into()),
    );
    std::fs::create_dir_all(&data_dir)?;
    let db_url = format!("sqlite://{}", data_dir.join("sentinel.db").display());
    let opts = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        // Azure Files can retain a SQLite lock briefly while a one-replica
        // revision is replaced. Wait for that hand-off instead of treating it
        // as a fatal startup failure.
        .busy_timeout(Duration::from_secs(30))
        .foreign_keys(true);
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await?;
    sqlx::migrate!().run(&db).await?;
    let master_key_source = key_source(&data_dir);
    let secrets = SecretBox::from_data_dir(&data_dir)?;
    migrate_plaintext_canaries(&db, &secrets).await?;
    let (access_token, access_token_source) = access_token(&data_dir)?;
    let state = AppState {
        db,
        secrets,
        access_token,
        allow_private_endpoints: false,
    };
    tokio::spawn(scheduler(state.clone()));
    let static_dir_supplied = std::env::var_os("STATIC_DIR").is_some();
    let default_static_dir = if Path::new("/app/dist").is_dir() {
        "/app/dist"
    } else {
        "dist"
    };
    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| default_static_dir.into());
    let app = router(state, Path::new(&static_dir));
    let port_supplied = std::env::var_os("PORT").is_some();
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    tracing::info!(
        event = "startup_configuration",
        data_dir_source = if data_dir_supplied {
            "supplied"
        } else {
            "default"
        },
        static_dir_source = if static_dir_supplied {
            "supplied"
        } else {
            "default"
        },
        port_source = if port_supplied { "supplied" } else { "default" },
        master_key_source,
        access_token_source,
        build = routes::build_identity(),
        "startup configuration sources"
    );
    tracing::info!(port=%port,"capacity sentinel listening");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown())
    .await?;
    Ok(())
}

async fn migrate_plaintext_canaries(db: &SqlitePool, secrets: &SecretBox) -> anyhow::Result<()> {
    sqlx::query("PRAGMA secure_delete=ON").execute(db).await?;
    let rows: Vec<(String, String)> =
        sqlx::query_as("SELECT id, prompt FROM probes WHERE prompt_cipher='' AND prompt<>''")
            .fetch_all(db)
            .await?;
    let migrated = !rows.is_empty();
    for (id, prompt) in rows {
        let encrypted = secrets.encrypt(&prompt)?;
        sqlx::query("UPDATE probes SET prompt='', prompt_cipher=? WHERE id=?")
            .bind(encrypted)
            .bind(id)
            .execute(db)
            .await?;
    }
    // A pre-repair database can have text in its main file or WAL.  After
    // replacing every legacy value, compact and truncate the WAL so the old
    // canary bytes are not retained as historical pages.
    if migrated {
        sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
            .execute(db)
            .await?;
        sqlx::query("VACUUM").execute(db).await?;
        sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
            .execute(db)
            .await?;
    }
    Ok(())
}

fn access_token(data_dir: &Path) -> anyhow::Result<(Arc<str>, &'static str)> {
    if let Ok(token) = std::env::var("SENTINEL_ACCESS_TOKEN") {
        if token.len() < 24 {
            anyhow::bail!("SENTINEL_ACCESS_TOKEN must be at least 24 characters");
        }
        return Ok((Arc::from(token), "supplied"));
    }
    let path = data_dir.join("access.token");
    if let Ok(token) = std::fs::read_to_string(&path) {
        return Ok((Arc::from(token.trim()), "persisted"));
    }
    use rand::RngCore;
    let mut bytes = [0_u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let token = base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, bytes);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)?;
        use std::io::Write;
        file.write_all(token.as_bytes())?;
    }
    #[cfg(not(unix))]
    std::fs::write(&path, &token)?;
    Ok((Arc::from(token), "generated"))
}

fn router(state: AppState, static_dir: &Path) -> Router {
    let index = static_dir.join("index.html");
    let fallback = ServeDir::new(static_dir)
        .not_found_service(ServeFile::new(static_dir.join("404.html")));
    // General API traffic gets a 20 req/s, burst-40 budget. Mutating routes
    // additionally get a stricter 4 req/s, burst-20 budget. Both budgets are
    // per client at the trusted ingress boundary.
    let api_limit = GovernorConfigBuilder::default()
        .per_millisecond(50)
        .burst_size(40)
        .key_extractor(ClientIp)
        .finish()
        .expect("valid API rate-limit configuration");
    let write_limit = GovernorConfigBuilder::default()
        .per_millisecond(250)
        .burst_size(20)
        .methods(vec![
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
        ])
        .key_extractor(ClientIp)
        .finish()
        .expect("valid write rate-limit configuration");
    let api = Router::new()
        .route("/api/summary", get(routes::summary))
        .route("/api/probes", post(routes::create))
        .route(
            "/api/probes/{id}",
            put(routes::update).delete(routes::remove),
        )
        .route("/api/probes/{id}/run", post(routes::run))
        .route("/api/observations", get(routes::history))
        .route("/api/export.csv", get(routes::export))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_access,
        ))
        .layer(GovernorLayer::new(write_limit).error_handler(rate_limit_response))
        .layer(GovernorLayer::new(api_limit).error_handler(rate_limit_response));
    Router::new()
        .route_service("/", ServeFile::new(&index))
        .route_service("/demo", ServeFile::new(&index))
        .route_service("/privacy", ServeFile::new(&index))
        .route_service("/terms", ServeFile::new(&index))
        .route("/health", get(routes::health))
        .merge(api)
        .fallback_service(fallback)
        .with_state(state)
        .layer(middleware::from_fn(security_headers))
        .layer(middleware::from_fn(cache_control))
        .layer(RequestBodyLimitLayer::new(64 * 1024))
        .layer(CompressionLayer::new())
        .layer(CatchPanicLayer::new())
        .layer(TraceLayer::new_for_http())
}

/// Resolve the client at the factory ingress boundary. The ingress supplies
/// `X-Forwarded-For`; its first hop is the originating client. Native and
/// self-hosted requests safely fall back to axum's transport peer address.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ClientIp;

impl KeyExtractor for ClientIp {
    type Key = IpAddr;

    fn name(&self) -> &'static str {
        "first forwarded or peer IP"
    }

    fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, GovernorError> {
        if let Some(forwarded) = req.headers().get("x-forwarded-for") {
            let first = forwarded
                .to_str()
                .ok()
                .and_then(|value| value.split(',').next())
                .map(str::trim)
                .and_then(|value| value.parse::<IpAddr>().ok())
                .ok_or(GovernorError::UnableToExtractKey)?;
            return Ok(first);
        }

        req.extensions()
            .get::<axum::extract::ConnectInfo<SocketAddr>>()
            .map(|peer| peer.0.ip())
            .or_else(|| req.extensions().get::<SocketAddr>().map(SocketAddr::ip))
            .ok_or(GovernorError::UnableToExtractKey)
    }

    fn key_name(&self, key: &Self::Key) -> Option<String> {
        Some(key.to_string())
    }
}

fn rate_limit_response(error: GovernorError) -> axum::http::Response<Body> {
    match error {
        GovernorError::TooManyRequests { wait_time, headers } => {
            // Retry-After uses whole seconds. Governor reports a floored value,
            // so round up to avoid inviting a retry before a token is ready.
            let retry_after = wait_time.saturating_add(1).max(1);
            let mut response = (
                axum::http::StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({
                    "error": "Too many requests; retry after the indicated delay"
                })),
            )
                .into_response();
            if let Some(headers) = headers {
                response.headers_mut().extend(headers);
            }
            response.headers_mut().insert(
                header::RETRY_AFTER,
                HeaderValue::from_str(&retry_after.to_string())
                    .expect("integer Retry-After is a valid header"),
            );
            response.headers_mut().insert(
                header::HeaderName::from_static("x-ratelimit-after"),
                HeaderValue::from_str(&retry_after.to_string())
                    .expect("integer reset delay is a valid header"),
            );
            response
        }
        GovernorError::UnableToExtractKey => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error":"Unable to identify request client"})),
        )
            .into_response(),
        GovernorError::Other { code, msg, headers } => {
            let mut response = (
                code,
                Json(serde_json::json!({
                    "error": msg.unwrap_or_else(|| "Rate limiter error".into())
                })),
            )
                .into_response();
            if let Some(headers) = headers {
                response.headers_mut().extend(headers);
            }
            response
        }
    }
}

async fn require_access(
    axum::extract::State(state): axum::extract::State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    use subtle::ConstantTimeEq;
    let supplied = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    let valid =
        supplied.is_some_and(|token| token.as_bytes().ct_eq(state.access_token.as_bytes()).into());
    if valid {
        next.run(req).await
    } else {
        (
            axum::http::StatusCode::UNAUTHORIZED,
            [(header::WWW_AUTHENTICATE, "Bearer")],
            Json(serde_json::json!({"error":"A project access code is required"})),
        )
            .into_response()
    }
}

async fn cache_control(req: Request<Body>, next: Next) -> Response {
    let path = req.uri().path().to_owned();
    let mut res = next.run(req).await;
    let value = if path.starts_with("/api/") || path == "/health" {
        "no-store"
    } else if path.starts_with("/assets/index-") {
        "public, max-age=31536000, immutable"
    } else if path == "/sw.js"
        || path == "/"
        || path == "/privacy"
        || path == "/terms"
        || path == "/index.html"
    {
        "no-cache"
    } else {
        "public, max-age=86400"
    };
    res.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static(value));
    res
}

async fn security_headers(req: Request<Body>, next: Next) -> Response {
    let mut res = next.run(req).await;
    let h = res.headers_mut();
    h.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    h.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    h.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    h.insert(header::CONTENT_SECURITY_POLICY,HeaderValue::from_static("default-src 'self'; img-src 'self' data:; style-src 'self'; connect-src 'self' https://api.sociobot.in; script-src 'self'; worker-src 'self'; base-uri 'none'; frame-ancestors 'none'; form-action 'self' https://api.sociobot.in"));
    res
}

async fn scheduler(state: AppState) {
    let mut ticker = tokio::time::interval(Duration::from_secs(30));
    loop {
        ticker.tick().await;
        let due:Vec<models::ProbeRow>=sqlx::query_as("SELECT p.* FROM probes p LEFT JOIN (SELECT probe_id,MAX(started_at) last_at FROM observations GROUP BY probe_id) o ON o.probe_id=p.id WHERE p.enabled=1 AND (o.last_at IS NULL OR datetime(o.last_at) <= datetime('now', '-' || p.interval_minutes || ' minutes'))").fetch_all(&state.db).await.unwrap_or_default();
        for p in due {
            let s = state.clone();
            tokio::spawn(async move {
                probe::execute(&s, &p).await;
            });
        }
    }
}
async fn shutdown() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("install Ctrl+C handler")
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {_=ctrl_c=>{},_=terminate=>{}}
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::{models::ProbeInput, routes::resolve_public_endpoint};
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
        routing::post,
        Json,
    };
    use serde_json::{json, Value};
    use tower::ServiceExt;

    async fn test_state(dir: &std::path::Path) -> AppState {
        let options =
            SqliteConnectOptions::from_str(&format!("sqlite://{}", dir.join("test.db").display()))
                .unwrap()
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal)
                .foreign_keys(true);
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        AppState {
            db,
            secrets: SecretBox::from_data_dir(dir).unwrap(),
            access_token: Arc::from("test-access-token-which-is-long-enough"),
            allow_private_endpoints: true,
        }
    }
    fn json_request(method: &str, uri: &str, body: Value) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .header(
                "authorization",
                "Bearer test-access-token-which-is-long-enough",
            )
            .header("x-forwarded-for", "192.0.2.10, 10.0.0.5")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    fn authenticated_request(method: &str, uri: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header(
                "authorization",
                "Bearer test-access-token-which-is-long-enough",
            )
            .header("x-forwarded-for", "192.0.2.10, 10.0.0.5")
            .body(Body::empty())
            .unwrap()
    }

    // @claim:encrypted-storage-and-classification
    #[tokio::test]
    async fn claim_encrypted_storage_and_failure_attribution() {
        let mock = Router::new().route(
            "/chat",
            post(|| async {
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(json!({"error":"capacity"})),
                )
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, mock).await.unwrap() });
        let temp = tempfile::tempdir().unwrap();
        let state = test_state(temp.path()).await;
        let app = router(state.clone(), temp.path());
        let input = json!({"name":"Capacity canary","provider":"Test provider","endpoint_url":format!("http://{address}/chat"),"model":"test-model","api_key":"secret-value","prompt":"Return JSON","required_fields":["status"],"interval_minutes":5,"timeout_ms":2000,"latency_slo_ms":1000,"availability_slo_percent":99,"max_output_tokens":32,"daily_token_cap":1000,"enabled":true});
        let response = app
            .clone()
            .oneshot(json_request("POST", "/api/probes", input.clone()))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let id: String = sqlx::query_scalar("SELECT id FROM probes LIMIT 1")
            .fetch_one(&state.db)
            .await
            .unwrap();
        let cipher: String = sqlx::query_scalar("SELECT api_key_cipher FROM probes WHERE id=?")
            .bind(&id)
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert!(!cipher.contains("secret-value"));
        let stored_prompt: String = sqlx::query_scalar("SELECT prompt FROM probes WHERE id=?")
            .bind(&id)
            .fetch_one(&state.db)
            .await
            .unwrap();
        let prompt_cipher: String =
            sqlx::query_scalar("SELECT prompt_cipher FROM probes WHERE id=?")
                .bind(&id)
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert!(stored_prompt.is_empty());
        assert!(!prompt_cipher.contains("Return JSON"));
        for _ in 0..2 {
            let response = app
                .clone()
                .oneshot(json_request(
                    "POST",
                    &format!("/api/probes/{id}/run"),
                    json!({}),
                ))
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
        let response = app
            .clone()
            .oneshot(authenticated_request("GET", "/api/summary"))
            .await
            .unwrap();
        let body = to_bytes(response.into_body(), 100_000).await.unwrap();
        let summary: Value = serde_json::from_slice(&body).unwrap();
        assert!(summary["probes"][0].get("prompt").is_none());
        assert_eq!(summary["alerts"][0]["kind"], "availability");
        assert_eq!(summary["observations"][0]["error_class"], "capacity");
        let mut updated = input;
        updated["name"] = json!("Updated canary");
        updated["api_key"] = json!("");
        assert_eq!(
            app.clone()
                .oneshot(json_request("PUT", &format!("/api/probes/{id}"), updated))
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
        assert_eq!(
            app.clone()
                .oneshot(authenticated_request("GET", "/api/observations"))
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
        assert_eq!(
            app.clone()
                .oneshot(authenticated_request("GET", "/api/export.csv"))
                .await
                .unwrap()
                .status(),
            StatusCode::OK
        );
        assert_eq!(
            app.clone()
                .oneshot(
                    Request::delete(format!("/api/probes/{id}"))
                        .header(
                            "authorization",
                            "Bearer test-access-token-which-is-long-enough"
                        )
                        .header("x-forwarded-for", "192.0.2.10, 10.0.0.5")
                        .body(Body::empty())
                        .unwrap()
                )
                .await
                .unwrap()
                .status(),
            StatusCode::NO_CONTENT
        );
        let response = app
            .oneshot(Request::get("/health").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = to_bytes(response.into_body(), 1_000).await.unwrap();
        let health: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(health["build"], routes::build_identity());
        assert_ne!(health["build"], "unknown");
    }

    // @claim:public-endpoint-safety
    #[tokio::test]
    async fn claim_public_endpoint_safety() {
        let temp = tempfile::tempdir().unwrap();
        let state = test_state(temp.path()).await;
        let app = router(state, temp.path());
        assert_eq!(
            app.clone()
                .oneshot(
                    Request::get("/api/summary")
                        .header("x-forwarded-for", "192.0.2.10")
                        .body(Body::empty())
                        .unwrap()
                )
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        let input = ProbeInput {
            name: "x".into(),
            provider: "x".into(),
            endpoint_url: "http://127.0.0.1:8080/chat".into(),
            model: "x".into(),
            api_key: "x".into(),
            prompt: "synthetic".into(),
            required_fields: vec![],
            interval_minutes: 5,
            timeout_ms: 1000,
            latency_slo_ms: 100,
            availability_slo_percent: 99.0,
            max_output_tokens: 1,
            daily_token_cap: 1,
            enabled: true,
        };
        assert!(
            resolve_public_endpoint(&url::Url::parse(&input.endpoint_url).unwrap(), false)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn unauthenticated_api_attempts_are_limited_before_access_validation() {
        let temp = tempfile::tempdir().unwrap();
        let app = router(test_state(temp.path()).await, temp.path());
        let mut requests = tokio::task::JoinSet::new();
        for later_proxy_hop in 1..=41 {
            let app = app.clone();
            requests.spawn(async move {
                app.oneshot(
                    Request::get("/api/summary")
                        .header("authorization", "Bearer deliberately-wrong")
                        .header(
                            "x-forwarded-for",
                            format!("198.51.100.60, 10.0.0.{later_proxy_hop}"),
                        )
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap()
            });
        }
        let mut unauthorized = 0;
        let mut limited = Vec::new();
        while let Some(result) = requests.join_next().await {
            let response = result.unwrap();
            if response.status() == StatusCode::TOO_MANY_REQUESTS {
                limited.push(response);
            } else {
                assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
                unauthorized += 1;
            }
        }
        assert_eq!(unauthorized, 40);
        assert_eq!(limited.len(), 1);
        assert!(limited[0]
            .headers()
            .get(header::RETRY_AFTER)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
            .is_some_and(|seconds| seconds >= 1));
    }

    #[tokio::test]
    async fn api_read_limit_uses_first_forwarded_ip_and_returns_retry_after() {
        let temp = tempfile::tempdir().unwrap();
        let app = router(test_state(temp.path()).await, temp.path());
        let mut requests = tokio::task::JoinSet::new();
        for second_hop in 1..=41 {
            let app = app.clone();
            requests.spawn(async move {
                app.oneshot(
                    Request::get("/api/summary")
                        .header(
                            "authorization",
                            "Bearer test-access-token-which-is-long-enough",
                        )
                        .header(
                            "x-forwarded-for",
                            format!("198.51.100.10, 10.0.0.{second_hop}"),
                        )
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap()
            });
        }
        let mut ok = 0;
        let mut limited = Vec::new();
        while let Some(result) = requests.join_next().await {
            let response = result.unwrap();
            if response.status() == StatusCode::TOO_MANY_REQUESTS {
                limited.push(response);
            } else {
                assert_eq!(response.status(), StatusCode::OK);
                ok += 1;
            }
        }
        assert_eq!(ok, 40);
        assert_eq!(limited.len(), 1);
        let retry_after = limited[0]
            .headers()
            .get(header::RETRY_AFTER)
            .expect("429 must include Retry-After")
            .to_str()
            .unwrap()
            .parse::<u64>()
            .unwrap();
        assert!(retry_after >= 1);

        // All 41 later proxy hops were distinct, so the exhausted shared
        // quota proves only the first forwarded hop is used. A different
        // originating client still has an independent budget.
        let other_client = app
            .oneshot(
                Request::get("/api/summary")
                    .header(
                        "authorization",
                        "Bearer test-access-token-which-is-long-enough",
                    )
                    .header("x-forwarded-for", "198.51.100.11, 10.0.0.1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(other_client.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn api_write_limit_is_stricter_per_client_and_returns_retry_after() {
        let temp = tempfile::tempdir().unwrap();
        let app = router(test_state(temp.path()).await, temp.path());
        let mut requests = tokio::task::JoinSet::new();
        for _ in 0..21 {
            let app = app.clone();
            requests.spawn(async move {
                app.oneshot(
                    Request::post("/api/probes")
                        .header(
                            "authorization",
                            "Bearer test-access-token-which-is-long-enough",
                        )
                        .header("content-type", "application/json")
                        .header("x-forwarded-for", "203.0.113.20, 10.0.0.5")
                        .body(Body::from("{}"))
                        .unwrap(),
                )
                .await
                .unwrap()
            });
        }
        let mut validation_errors = 0;
        let mut limited = Vec::new();
        while let Some(result) = requests.join_next().await {
            let response = result.unwrap();
            if response.status() == StatusCode::TOO_MANY_REQUESTS {
                limited.push(response);
            } else {
                assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
                validation_errors += 1;
            }
        }
        assert_eq!(validation_errors, 20);
        assert_eq!(limited.len(), 1);
        assert!(limited[0].headers().contains_key(header::RETRY_AFTER));

        let other_client = app
            .oneshot(
                Request::post("/api/probes")
                    .header(
                        "authorization",
                        "Bearer test-access-token-which-is-long-enough",
                    )
                    .header("content-type", "application/json")
                    .header("x-forwarded-for", "203.0.113.21, 10.0.0.5")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(other_client.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn client_ip_falls_back_to_transport_peer_without_forwarded_header() {
        let mut request = Request::new(Body::empty());
        request.extensions_mut().insert(axum::extract::ConnectInfo(
            "192.0.2.44:43123".parse::<SocketAddr>().unwrap(),
        ));
        assert_eq!(
            ClientIp.extract(&request).unwrap(),
            "192.0.2.44".parse::<IpAddr>().unwrap()
        );
    }

    #[tokio::test]
    async fn legacy_plaintext_canary_is_encrypted_and_scrubbed_on_startup() {
        let temp = tempfile::tempdir().unwrap();
        let state = test_state(temp.path()).await;
        sqlx::query("INSERT INTO probes(id,name,provider,endpoint_url,model,api_key_cipher,prompt,prompt_cipher,required_fields,interval_minutes,timeout_ms,latency_slo_ms,availability_slo_percent,max_output_tokens,daily_token_cap,enabled,created_at,updated_at) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)")
            .bind("legacy").bind("Legacy").bind("Provider").bind("https://example.com/chat").bind("model")
            .bind(state.secrets.encrypt("key").unwrap()).bind("Synthetic QA capacity only").bind("").bind("[]")
            .bind(5).bind(1000).bind(100).bind(99.0).bind(1).bind(10).bind(1).bind("2026-01-01T00:00:00Z").bind("2026-01-01T00:00:00Z")
            .execute(&state.db).await.unwrap();
        migrate_plaintext_canaries(&state.db, &state.secrets)
            .await
            .unwrap();
        let (plain, cipher): (String, String) =
            sqlx::query_as("SELECT prompt,prompt_cipher FROM probes WHERE id='legacy'")
                .fetch_one(&state.db)
                .await
                .unwrap();
        assert!(plain.is_empty());
        assert!(!cipher.contains("Synthetic QA capacity only"));
        assert_eq!(
            state.secrets.decrypt(&cipher).unwrap(),
            "Synthetic QA capacity only"
        );
    }
}
