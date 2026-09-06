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
use sqlx::{sqlite::{SqliteConnectOptions, SqlitePoolOptions}, SqlitePool};
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

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

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
    let (db, database_source) = open_database(&data_dir).await?;
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
        database_source,
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

async fn open_database(data_dir: &Path) -> anyhow::Result<(SqlitePool, &'static str)> {
    let primary_path = data_dir.join("sentinel.db");
    let recovery_path = data_dir.join("sentinel-recovery.db");
    if recovery_path.is_file() {
        let recovery = connect_database(&recovery_path).await?;
        migrate_database(&recovery).await?;
        return Ok((recovery, "recovery"));
    }

    let primary = connect_database(&primary_path).await?;
    let primary_needs_initialization = !migrations_table_exists(&primary).await?;
    match migrate_database(&primary).await {
        Ok(()) => Ok((primary, "primary")),
        Err(error) if primary_needs_initialization && database_is_locked(&error) => {
            tracing::warn!(
                event = "database_recovery",
                reason = "locked_empty_primary",
                "preserving locked empty primary database and initializing recovery database"
            );
            primary.close().await;
            let recovery = connect_database(&recovery_path).await?;
            migrate_database(&recovery).await?;
            Ok((recovery, "recovery"))
        }
        Err(error) => Err(error),
    }
}

async fn connect_database(path: &Path) -> anyhow::Result<SqlitePool> {
    // Azure Files does not reliably release SQLite's default POSIX byte-range
    // locks across container restarts. The built-in dot-file VFS coordinates
    // through an atomic directory on the mounted filesystem instead. Keep one
    // connection because dot-file locking deliberately serializes all access;
    // this matches the product's required one-replica SQLite topology.
    let db_url = format!("sqlite://{}?vfs=unix-dotfile", path.display());
    let options = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true)
        // A previous one-replica process can retain the filesystem lock
        // briefly while the durable Azure Files mount is handed over. Wait
        // for that hand-off instead of failing immediately. Do not issue a
        // `PRAGMA journal_mode` here: switching journal modes requires an
        // exclusive lock that SQLite cannot wait for with busy_timeout.
        .busy_timeout(Duration::from_secs(30))
        .foreign_keys(true);
    Ok(SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await?)
}

fn database_is_locked(error: &anyhow::Error) -> bool {
    error
        .chain()
        .any(|cause| cause.to_string().contains("database is locked"))
}

/// Validate an already-current schema without issuing SQLite's write-oriented
/// `CREATE TABLE IF NOT EXISTS` migration prelude. That prelude can stay
/// locked briefly on Azure Files after a one-replica handoff even when no
/// migration is pending.
async fn migrate_database(db: &SqlitePool) -> anyhow::Result<()> {
    if schema_is_current(db).await? {
        tracing::info!("database schema is current");
        return Ok(());
    }

    MIGRATOR.run(db).await?;
    Ok(())
}

async fn schema_is_current(db: &SqlitePool) -> anyhow::Result<bool> {
    if !migrations_table_exists(db).await? {
        tracing::info!(event = "schema_validation", reason = "migration_table_missing");
        return Ok(false);
    }

    let applied: Vec<(i64, Vec<u8>)> =
        sqlx::query_as("SELECT version, checksum FROM _sqlx_migrations WHERE success = TRUE")
            .fetch_all(db)
            .await?;
    let expected: Vec<_> = MIGRATOR
        .iter()
        .filter(|migration| migration.migration_type.is_up_migration())
        .collect();
    let matches = applied.len() == expected.len()
        && expected.iter().all(|migration| {
            applied.iter().any(|(version, checksum)| {
                *version == migration.version && checksum.as_slice() == migration.checksum.as_ref()
            })
        });
    tracing::info!(
        event = "schema_validation",
        applied_migration_count = applied.len(),
        expected_migration_count = expected.len(),
        matches,
    );
    Ok(matches)
}

async fn migrations_table_exists(db: &SqlitePool) -> anyhow::Result<bool> {
    let exists: i64 = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations')",
    )
    .fetch_one(db)
    .await?;
    Ok(exists != 0)
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
    use crate::routes::resolve_public_endpoint;
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
        routing::post,
        Json,
    };
    use serde_json::{json, Value};
    use std::{
        process::{Command, Stdio},
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc as StdArc, Mutex,
        },
        thread,
        time::Instant,
    };
    use tower::ServiceExt;

    async fn test_state(dir: &std::path::Path) -> AppState {
        let options =
            SqliteConnectOptions::from_str(&format!("sqlite://{}", dir.join("test.db").display()))
                .unwrap()
                .create_if_missing(true)
                .foreign_keys(true);
        let db = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();
        migrate_database(&db).await.unwrap();
        AppState {
            db,
            secrets: SecretBox::from_data_dir(dir).unwrap(),
            access_token: Arc::from("test-access-token-which-is-long-enough"),
            allow_private_endpoints: true,
        }
    }

    #[tokio::test]
    async fn current_schema_validation_does_not_require_a_database_write() {
        let dir = tempfile::tempdir().unwrap();
        let state = test_state(dir.path()).await;
        sqlx::query("PRAGMA query_only = ON")
            .execute(&state.db)
            .await
            .unwrap();

        migrate_database(&state.db).await.unwrap();
    }

    #[tokio::test]
    async fn schema_initializes_when_default_sqlite_locking_is_unavailable() {
        let temp = tempfile::tempdir().unwrap();
        let database = temp.path().join("remote-mount.db");
        let ready = temp.path().join("lock-ready");
        let mut lock_holder = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "integration_tests::conventional_sqlite_lock_helper",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("SENTINEL_LOCK_TEST_DB", &database)
            .env("SENTINEL_LOCK_TEST_READY", &ready)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();

        let deadline = Instant::now() + Duration::from_secs(5);
        while !ready.is_file() && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(25));
        }
        assert!(
            ready.is_file(),
            "helper did not acquire the conventional lock"
        );

        let default_options =
            SqliteConnectOptions::from_str(&format!("sqlite://{}", database.display()))
                .unwrap()
                .create_if_missing(true)
                .busy_timeout(Duration::from_millis(100));
        let default_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(default_options)
            .await
            .unwrap();
        let blocked = sqlx::query("CREATE TABLE blocked_by_remote_lock(id INTEGER)")
            .execute(&default_pool)
            .await;
        let lock_error = blocked.expect_err(
            "the fixture must reproduce the conventional SQLite lock failure",
        );
        assert!(lock_error.to_string().contains("database is locked"));
        default_pool.close().await;

        let compatible_pool = connect_database(&database).await.unwrap();
        migrate_database(&compatible_pool).await.unwrap();
        let applied: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE success = TRUE")
                .fetch_one(&compatible_pool)
                .await
                .unwrap();
        assert_eq!(applied, MIGRATOR.iter().count() as i64);
        compatible_pool.close().await;
        assert!(!PathBuf::from(format!("{}.lock", database.display())).exists());

        lock_holder.kill().ok();
        lock_holder.wait().unwrap();
    }

    #[tokio::test]
    async fn conventional_sqlite_lock_helper() {
        let (Ok(database), Ok(ready)) = (
            std::env::var("SENTINEL_LOCK_TEST_DB"),
            std::env::var("SENTINEL_LOCK_TEST_READY"),
        ) else {
            return;
        };
        let options = SqliteConnectOptions::from_str(&format!("sqlite://{database}"))
            .unwrap()
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();
        sqlx::query("BEGIN EXCLUSIVE").execute(&pool).await.unwrap();
        std::fs::write(ready, b"ready").unwrap();
        tokio::time::sleep(Duration::from_secs(30)).await;
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

    fn probe_input(endpoint: impl Into<String>) -> Value {
        json!({
            "name": "Capacity canary",
            "provider": "Test provider",
            "endpoint_url": endpoint.into(),
            "model": "test-model",
            "api_key": "test-api-key",
            "prompt": "Return a synthetic JSON status",
            "required_fields": ["status"],
            "interval_minutes": 5,
            "timeout_ms": 2000,
            "latency_slo_ms": 1000,
            "availability_slo_percent": 99,
            "max_output_tokens": 32,
            "daily_token_cap": 1000,
            "enabled": true
        })
    }

    async fn create_probe(
        app: &Router,
        state: &AppState,
        input: Value,
    ) -> String {
        let response = app
            .clone()
            .oneshot(json_request("POST", "/api/probes", input))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        sqlx::query_scalar("SELECT id FROM probes ORDER BY created_at DESC LIMIT 1")
            .fetch_one(&state.db)
            .await
            .unwrap()
    }

    async fn run_probe(app: &Router, id: &str) -> Value {
        let response = app
            .clone()
            .oneshot(json_request("POST", &format!("/api/probes/{id}/run"), json!({})))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        serde_json::from_slice(&to_bytes(response.into_body(), 100_000).await.unwrap()).unwrap()
    }

    fn successful_completion(content: &str) -> Json<Value> {
        Json(json!({
            "choices": [{"message": {"content": content}}],
            "usage": {"prompt_tokens": 3, "completion_tokens": 5}
        }))
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
        for endpoint in [
            "http://127.0.0.1:8080/chat",
            "http://10.0.0.1/chat",
            "http://169.254.1.1/chat",
            "http://192.168.1.1/chat",
            "http://[::1]/chat",
            "http://[fe80::1]/chat",
        ] {
            assert!(
                resolve_public_endpoint(&url::Url::parse(endpoint).unwrap(), false)
                    .await
                    .is_err(),
                "{endpoint} must be rejected before a probe can be saved"
            );
        }
    }

    // @claim:scheduled-probes
    #[tokio::test]
    async fn claim_scheduled_probes_create_observations_for_enabled_probes() {
        let mock = Router::new().route(
            "/chat",
            post(|| async { successful_completion(r#"{"status":"ok"}"#) }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, mock).await.unwrap() });

        let temp = tempfile::tempdir().unwrap();
        let state = test_state(temp.path()).await;
        let app = router(state.clone(), temp.path());
        let id = create_probe(&app, &state, probe_input(format!("http://{address}/chat"))).await;
        let scheduler_task = tokio::spawn(scheduler(state.clone()));
        tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                let count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM observations WHERE probe_id=?",
                )
                .bind(&id)
                .fetch_one(&state.db)
                .await
                .unwrap();
                if count == 1 {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
        })
        .await
        .expect("an enabled probe should run on the scheduler without a manual request");
        scheduler_task.abort();

        let outcome: String = sqlx::query_scalar(
            "SELECT outcome FROM observations WHERE probe_id=? LIMIT 1",
        )
        .bind(&id)
        .fetch_one(&state.db)
        .await
        .unwrap();
        assert_eq!(outcome, "healthy");
    }

    // @claim:real-monitoring-metrics
    #[tokio::test]
    async fn claim_real_monitoring_metrics_use_successful_endpoint_responses() {
        let mock = Router::new().route(
            "/chat",
            post(|| async {
                tokio::time::sleep(Duration::from_millis(30)).await;
                successful_completion(r#"{"status":"ok"}"#)
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, mock).await.unwrap() });

        let temp = tempfile::tempdir().unwrap();
        let state = test_state(temp.path()).await;
        let app = router(state.clone(), temp.path());
        let id = create_probe(&app, &state, probe_input(format!("http://{address}/chat"))).await;
        for _ in 0..3 {
            let observation = run_probe(&app, &id).await;
            assert_eq!(observation["outcome"], "healthy");
            assert!(observation["latency_ms"].as_i64().unwrap() >= 25);
        }
        let response = app
            .oneshot(authenticated_request("GET", "/api/summary"))
            .await
            .unwrap();
        let summary: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 100_000).await.unwrap()).unwrap();
        let stats = &summary["probes"][0]["stats"];
        assert_eq!(stats["sample_count"], 3);
        assert_eq!(stats["availability_percent"], 100.0);
        assert!(stats["p95_latency_ms"].as_i64().unwrap() >= 25);
    }

    // @claim:failure-classification
    #[tokio::test]
    async fn claim_failure_classification_reports_shape_timeout_network_and_upstream_errors() {
        let mock = Router::new()
            .route(
                "/invalid-json",
                post(|| async { successful_completion("not JSON") }),
            )
            .route(
                "/missing-field",
                post(|| async { successful_completion(r#"{"other":"value"}"#) }),
            )
            .route(
                "/upstream",
                post(|| async {
                    (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"error":"unavailable"})))
                }),
            )
            .route(
                "/slow",
                post(|| async {
                    tokio::time::sleep(Duration::from_millis(1_100)).await;
                    successful_completion(r#"{"status":"late"}"#)
                }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, mock).await.unwrap() });

        let temp = tempfile::tempdir().unwrap();
        let state = test_state(temp.path()).await;
        let app = router(state.clone(), temp.path());
        let cases = [
            ("invalid-json", "invalid_json"),
            ("missing-field", "invariant"),
            ("upstream", "upstream"),
            ("slow", "timeout"),
        ];
        for (path, expected) in cases {
            let mut input = probe_input(format!("http://{address}/{path}"));
            if path == "slow" {
                input["timeout_ms"] = json!(1000);
            }
            let id = create_probe(&app, &state, input).await;
            let observation = run_probe(&app, &id).await;
            assert_eq!(observation["error_class"], expected);
        }

        let closed = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let closed_address = closed.local_addr().unwrap();
        drop(closed);
        let id = create_probe(
            &app,
            &state,
            probe_input(format!("http://{closed_address}/unreachable")),
        )
        .await;
        let observation = run_probe(&app, &id).await;
        assert_eq!(observation["error_class"], "network");
    }

    // @claim:attributed-alert-recovery
    #[tokio::test]
    async fn claim_attributed_alerts_open_after_failures_and_resolve_after_recovery() {
        let healthy = StdArc::new(AtomicBool::new(false));
        let mock = Router::new().route(
            "/chat",
            post({
                let healthy = healthy.clone();
                move || {
                    let healthy = healthy.clone();
                    async move {
                        if healthy.load(Ordering::SeqCst) {
                            successful_completion(r#"{"status":"ok"}"#).into_response()
                        } else {
                            (StatusCode::TOO_MANY_REQUESTS, Json(json!({"error":"capacity"})))
                                .into_response()
                        }
                    }
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, mock).await.unwrap() });

        let temp = tempfile::tempdir().unwrap();
        let state = test_state(temp.path()).await;
        let app = router(state.clone(), temp.path());
        let mut input = probe_input(format!("http://{address}/chat"));
        input["provider"] = json!("Northern provider");
        input["model"] = json!("model-amber");
        let id = create_probe(&app, &state, input).await;
        run_probe(&app, &id).await;
        run_probe(&app, &id).await;
        let response = app
            .clone()
            .oneshot(authenticated_request("GET", "/api/summary"))
            .await
            .unwrap();
        let opened: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 100_000).await.unwrap()).unwrap();
        assert_eq!(opened["alerts"][0]["status"], "open");
        let message = opened["alerts"][0]["message"].as_str().unwrap();
        assert!(message.contains("Northern provider/model-amber"));
        assert!(message.contains("capacity"));

        healthy.store(true, Ordering::SeqCst);
        let recovered = run_probe(&app, &id).await;
        assert_eq!(recovered["outcome"], "healthy");
        let response = app
            .oneshot(authenticated_request("GET", "/api/summary"))
            .await
            .unwrap();
        let summary: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 100_000).await.unwrap()).unwrap();
        assert_eq!(summary["alerts"][0]["status"], "resolved");
        assert!(summary["alerts"][0]["resolved_at"].is_string());
    }

    // @claim:edit-preserves-history
    #[tokio::test]
    async fn claim_editing_a_probe_keeps_its_existing_observations() {
        let mock = Router::new().route(
            "/chat",
            post(|| async { successful_completion(r#"{"status":"ok"}"#) }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, mock).await.unwrap() });
        let temp = tempfile::tempdir().unwrap();
        let state = test_state(temp.path()).await;
        let app = router(state.clone(), temp.path());
        let input = probe_input(format!("http://{address}/chat"));
        let id = create_probe(&app, &state, input.clone()).await;
        run_probe(&app, &id).await;
        run_probe(&app, &id).await;

        let mut changed = input;
        changed["model"] = json!("test-model-revised");
        changed["api_key"] = json!("");
        changed["prompt"] = json!("");
        let response = app
            .clone()
            .oneshot(json_request("PUT", &format!("/api/probes/{id}"), changed))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let response = app
            .oneshot(authenticated_request(
                "GET",
                &format!("/api/observations?probe_id={id}"),
            ))
            .await
            .unwrap();
        let observations: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 100_000).await.unwrap()).unwrap();
        assert_eq!(observations.as_array().unwrap().len(), 2);
        assert!(observations
            .as_array()
            .unwrap()
            .iter()
            .all(|observation| observation["probe_id"] == id));
        let model: String = sqlx::query_scalar("SELECT model FROM probes WHERE id=?")
            .bind(&id)
            .fetch_one(&state.db)
            .await
            .unwrap();
        assert_eq!(model, "test-model-revised");
    }

    // @claim:token-limits
    #[tokio::test]
    async fn claim_token_limits_send_the_output_limit_and_block_the_daily_cap() {
        let requests = StdArc::new(Mutex::new(Vec::<Value>::new()));
        let mock = Router::new().route(
            "/chat",
            post({
                let requests = requests.clone();
                move |Json(body): Json<Value>| {
                    let requests = requests.clone();
                    async move {
                        requests.lock().unwrap().push(body);
                        successful_completion(r#"{"status":"ok"}"#)
                    }
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, mock).await.unwrap() });
        let temp = tempfile::tempdir().unwrap();
        let state = test_state(temp.path()).await;
        let app = router(state.clone(), temp.path());
        let mut input = probe_input(format!("http://{address}/chat"));
        input["max_output_tokens"] = json!(7);
        input["daily_token_cap"] = json!(20);
        let id = create_probe(&app, &state, input).await;
        let first = run_probe(&app, &id).await;
        assert_eq!(first["outcome"], "healthy");
        let second = run_probe(&app, &id).await;
        assert_eq!(second["error_class"], "cost_cap");
        assert_eq!(requests.lock().unwrap().len(), 1);
        assert_eq!(requests.lock().unwrap()[0]["max_tokens"], 7);
    }

    // @claim:probe-deletion
    #[tokio::test]
    async fn claim_deleting_a_probe_removes_its_settings_observations_and_alerts() {
        let mock = Router::new().route(
            "/chat",
            post(|| async { (StatusCode::TOO_MANY_REQUESTS, Json(json!({"error":"capacity"}))) }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, mock).await.unwrap() });
        let temp = tempfile::tempdir().unwrap();
        let state = test_state(temp.path()).await;
        let app = router(state.clone(), temp.path());
        let id = create_probe(&app, &state, probe_input(format!("http://{address}/chat"))).await;
        run_probe(&app, &id).await;
        run_probe(&app, &id).await;
        let response = app
            .clone()
            .oneshot(authenticated_request("DELETE", &format!("/api/probes/{id}")))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        let response = app
            .oneshot(authenticated_request("GET", "/api/summary"))
            .await
            .unwrap();
        let summary: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 100_000).await.unwrap()).unwrap();
        assert_eq!(summary["probes"], json!([]));
        assert_eq!(summary["observations"], json!([]));
        assert_eq!(summary["alerts"], json!([]));
    }

    // @claim:api-access-coverage
    #[tokio::test]
    async fn claim_every_project_api_route_requires_the_access_code() {
        let temp = tempfile::tempdir().unwrap();
        let app = router(test_state(temp.path()).await, temp.path());
        for (method, uri, body) in [
            ("GET", "/api/summary", Body::empty()),
            ("POST", "/api/probes", Body::from("{}")),
            ("PUT", "/api/probes/missing", Body::from("{}")),
            ("DELETE", "/api/probes/missing", Body::empty()),
            ("POST", "/api/probes/missing/run", Body::from("{}")),
            ("GET", "/api/observations", Body::empty()),
            ("GET", "/api/export.csv", Body::empty()),
        ] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method)
                        .uri(uri)
                        .header("content-type", "application/json")
                        .header("authorization", "Bearer wrong-access-code")
                        .header("x-forwarded-for", "198.51.100.211")
                        .body(body)
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{method} {uri}");
            assert_eq!(response.headers().get(header::WWW_AUTHENTICATE).unwrap(), "Bearer");
        }
    }

    // @claim:private-canary-boundary
    #[tokio::test]
    async fn claim_private_canary_data_reaches_only_the_configured_endpoint_and_is_not_retained() {
        let received = StdArc::new(Mutex::new(Vec::<(String, Value)>::new()));
        let mock = Router::new().route(
            "/chat",
            post({
                let received = received.clone();
                move |headers: axum::http::HeaderMap, Json(body): Json<Value>| {
                    let received = received.clone();
                    async move {
                        received.lock().unwrap().push((
                            headers
                                .get(header::AUTHORIZATION)
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .to_owned(),
                            body,
                        ));
                        successful_completion(r#"{"status":"provider-private-answer"}"#)
                    }
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, mock).await.unwrap() });
        let temp = tempfile::tempdir().unwrap();
        let state = test_state(temp.path()).await;
        let app = router(state.clone(), temp.path());
        let mut input = probe_input(format!("http://{address}/chat"));
        input["api_key"] = json!("private-api-key");
        input["prompt"] = json!("Synthetic private canary text");
        let id = create_probe(&app, &state, input).await;
        assert_eq!(run_probe(&app, &id).await["outcome"], "healthy");
        let received = received.lock().unwrap();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].0, "Bearer private-api-key");
        assert_eq!(received[0].1["messages"][0]["content"], "Synthetic private canary text");
        drop(received);

        let response = app
            .oneshot(authenticated_request("GET", "/api/summary"))
            .await
            .unwrap();
        let serialized = String::from_utf8(
            to_bytes(response.into_body(), 100_000).await.unwrap().to_vec(),
        )
        .unwrap();
        for private_value in [
            "private-api-key",
            "Synthetic private canary text",
            "provider-private-answer",
        ] {
            assert!(
                !serialized.contains(private_value),
                "operational records must not return {private_value}"
            );
        }
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
