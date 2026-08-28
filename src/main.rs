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
    response::Response,
    routing::{get, post, put},
    Router,
};
use crypto::SecretBox;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use std::{
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
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
    client: reqwest::Client,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let data_dir = PathBuf::from(std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".into()));
    std::fs::create_dir_all(&data_dir)?;
    let db_url = format!("sqlite://{}", data_dir.join("sentinel.db").display());
    let opts = SqliteConnectOptions::from_str(&db_url)?
        .create_if_missing(true)
        .foreign_keys(true);
    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await?;
    sqlx::migrate!().run(&db).await?;
    let state = AppState {
        db,
        secrets: SecretBox::from_data_dir(&data_dir)?,
        client: reqwest::Client::builder()
            .user_agent("Capacity-Sentinel/0.1")
            .build()?,
    };
    tokio::spawn(scheduler(state.clone()));
    let app = router(
        state,
        Path::new(&std::env::var("STATIC_DIR").unwrap_or_else(|_| "dist".into())),
    );
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
    tracing::info!(port=%port,"capacity sentinel listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}

fn router(state: AppState, static_dir: &Path) -> Router {
    let fallback =
        ServeDir::new(static_dir).not_found_service(ServeFile::new(static_dir.join("index.html")));
    Router::new()
        .route("/health", get(routes::health))
        .route("/api/summary", get(routes::summary))
        .route("/api/probes", post(routes::create))
        .route(
            "/api/probes/{id}",
            put(routes::update).delete(routes::remove),
        )
        .route("/api/probes/{id}/run", post(routes::run))
        .route("/api/observations", get(routes::history))
        .route("/api/export.csv", get(routes::export))
        .fallback_service(fallback)
        .with_state(state)
        .layer(middleware::from_fn(security_headers))
        .layer(RequestBodyLimitLayer::new(64 * 1024))
        .layer(CompressionLayer::new())
        .layer(CatchPanicLayer::new())
        .layer(TraceLayer::new_for_http())
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
    h.insert(header::CONTENT_SECURITY_POLICY,HeaderValue::from_static("default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; connect-src 'self' https://api.sociobot.in; script-src 'self'; base-uri 'none'; frame-ancestors 'none'; form-action 'self' https://api.sociobot.in"));
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
