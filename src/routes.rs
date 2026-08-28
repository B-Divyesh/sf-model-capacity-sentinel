use crate::{
    models::{Alert, Observation, ProbeInput, ProbeRow, ProbeView},
    probe::{calculate_stats, execute},
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(sqlx::FromRow)]
struct ExportRow {
    name: String,
    provider: String,
    model: String,
    started_at: String,
    latency_ms: i64,
    http_status: Option<i64>,
    outcome: String,
    error_class: Option<String>,
    detail: Option<String>,
    input_tokens: Option<i64>,
    output_tokens: Option<i64>,
}

pub type ApiResult<T> = Result<T, ApiError>;
pub struct ApiError(StatusCode, String);
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.0, Json(json!({"error":self.1}))).into_response()
    }
}
fn internal(e: impl std::fmt::Display) -> ApiError {
    tracing::error!(error=%e,"request failed");
    ApiError(
        StatusCode::INTERNAL_SERVER_ERROR,
        "The request could not be completed".into(),
    )
}

#[derive(Serialize)]
pub struct Summary {
    pub probes: Vec<ProbeView>,
    pub alerts: Vec<Alert>,
    pub observations: Vec<Observation>,
}

pub async fn health() -> Json<Value> {
    Json(json!({"status":"ok","build":option_env!("BUILD_SHA").unwrap_or("dev")}))
}

pub async fn summary(State(s): State<AppState>) -> ApiResult<Json<Summary>> {
    let probes: Vec<ProbeRow> = sqlx::query_as("SELECT * FROM probes ORDER BY created_at")
        .fetch_all(&s.db)
        .await
        .map_err(internal)?;
    let mut views = Vec::with_capacity(probes.len());
    for p in probes {
        views.push(view(&s, p).await?);
    }
    let alerts = sqlx::query_as("SELECT * FROM alerts ORDER BY opened_at DESC LIMIT 50")
        .fetch_all(&s.db)
        .await
        .map_err(internal)?;
    let observations = sqlx::query_as("SELECT * FROM observations WHERE started_at >= datetime('now','-365 days') ORDER BY started_at DESC LIMIT 5000")
            .fetch_all(&s.db)
            .await
            .map_err(internal)?;
    Ok(Json(Summary {
        probes: views,
        alerts,
        observations,
    }))
}

async fn view(s: &AppState, p: ProbeRow) -> ApiResult<ProbeView> {
    let rows: Vec<Observation> = sqlx::query_as(
        "SELECT * FROM observations WHERE probe_id=? ORDER BY started_at DESC LIMIT 20",
    )
    .bind(&p.id)
    .fetch_all(&s.db)
    .await
    .map_err(internal)?;
    let last_observation = rows.first().cloned();
    let stats = calculate_stats(&rows);
    let fields = p.fields();
    Ok(ProbeView {
        id: p.id,
        name: p.name,
        provider: p.provider,
        endpoint_url: p.endpoint_url,
        model: p.model,
        has_api_key: !p.api_key_cipher.is_empty(),
        prompt: p.prompt,
        required_fields: fields,
        interval_minutes: p.interval_minutes,
        timeout_ms: p.timeout_ms,
        latency_slo_ms: p.latency_slo_ms,
        availability_slo_percent: p.availability_slo_percent,
        max_output_tokens: p.max_output_tokens,
        daily_token_cap: p.daily_token_cap,
        enabled: p.enabled != 0,
        created_at: p.created_at,
        updated_at: p.updated_at,
        last_observation,
        stats,
    })
}

fn validate(i: &ProbeInput, editing: bool) -> ApiResult<()> {
    if i.name.trim().is_empty() || i.name.len() > 80 {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Name must be 1–80 characters".into(),
        ));
    }
    if i.provider.trim().is_empty() || i.provider.len() > 50 {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Provider is required".into(),
        ));
    }
    let url = url::Url::parse(&i.endpoint_url)
        .map_err(|_| ApiError(StatusCode::BAD_REQUEST, "Enter a valid endpoint URL".into()))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Endpoint must use HTTP or HTTPS".into(),
        ));
    }
    if i.model.trim().is_empty() || i.prompt.trim().is_empty() {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Model and synthetic prompt are required".into(),
        ));
    }
    if !editing && i.api_key.trim().is_empty() {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "API key is required".into(),
        ));
    }
    if !(1..=1440).contains(&i.interval_minutes)
        || !(1000..=120000).contains(&i.timeout_ms)
        || !(100..=120000).contains(&i.latency_slo_ms)
    {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Interval, timeout, or latency objective is outside the allowed range".into(),
        ));
    }
    if !(1.0..=100.0).contains(&i.availability_slo_percent)
        || !(1..=8192).contains(&i.max_output_tokens)
        || !(1..=10_000_000).contains(&i.daily_token_cap)
    {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "SLO or token budget is outside the allowed range".into(),
        ));
    }
    if i.required_fields.len() > 50 || i.required_fields.iter().any(|f| f.len() > 100) {
        return Err(ApiError(
            StatusCode::BAD_REQUEST,
            "Too many or overly long required fields".into(),
        ));
    }
    Ok(())
}

pub async fn create(
    State(s): State<AppState>,
    Json(i): Json<ProbeInput>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    validate(&i, false)?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let cipher = s.secrets.encrypt(&i.api_key).map_err(internal)?;
    sqlx::query("INSERT INTO probes VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)")
        .bind(&id)
        .bind(i.name.trim())
        .bind(i.provider.trim())
        .bind(i.endpoint_url.trim())
        .bind(i.model.trim())
        .bind(cipher)
        .bind(i.prompt.trim())
        .bind(serde_json::to_string(&i.required_fields).unwrap())
        .bind(i.interval_minutes)
        .bind(i.timeout_ms)
        .bind(i.latency_slo_ms)
        .bind(i.availability_slo_percent)
        .bind(i.max_output_tokens)
        .bind(i.daily_token_cap)
        .bind(i.enabled as i64)
        .bind(&now)
        .bind(&now)
        .execute(&s.db)
        .await
        .map_err(internal)?;
    Ok((StatusCode::CREATED, Json(json!({"id":id}))))
}

pub async fn update(
    Path(id): Path<String>,
    State(s): State<AppState>,
    Json(i): Json<ProbeInput>,
) -> ApiResult<Json<Value>> {
    validate(&i, true)?;
    let existing: Option<String> =
        sqlx::query_scalar("SELECT api_key_cipher FROM probes WHERE id=?")
            .bind(&id)
            .fetch_optional(&s.db)
            .await
            .map_err(internal)?;
    let old = existing.ok_or(ApiError(StatusCode::NOT_FOUND, "Probe not found".into()))?;
    let cipher = if i.api_key.trim().is_empty() {
        old
    } else {
        s.secrets.encrypt(&i.api_key).map_err(internal)?
    };
    sqlx::query("UPDATE probes SET name=?,provider=?,endpoint_url=?,model=?,api_key_cipher=?,prompt=?,required_fields=?,interval_minutes=?,timeout_ms=?,latency_slo_ms=?,availability_slo_percent=?,max_output_tokens=?,daily_token_cap=?,enabled=?,updated_at=? WHERE id=?")
        .bind(i.name.trim()).bind(i.provider.trim()).bind(i.endpoint_url.trim()).bind(i.model.trim()).bind(cipher).bind(i.prompt.trim()).bind(serde_json::to_string(&i.required_fields).unwrap()).bind(i.interval_minutes).bind(i.timeout_ms).bind(i.latency_slo_ms).bind(i.availability_slo_percent).bind(i.max_output_tokens).bind(i.daily_token_cap).bind(i.enabled as i64).bind(Utc::now().to_rfc3339()).bind(&id).execute(&s.db).await.map_err(internal)?;
    Ok(Json(json!({"id":id})))
}

pub async fn remove(Path(id): Path<String>, State(s): State<AppState>) -> ApiResult<StatusCode> {
    let n = sqlx::query("DELETE FROM probes WHERE id=?")
        .bind(id)
        .execute(&s.db)
        .await
        .map_err(internal)?
        .rows_affected();
    if n == 0 {
        Err(ApiError(StatusCode::NOT_FOUND, "Probe not found".into()))
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

pub async fn run(
    Path(id): Path<String>,
    State(s): State<AppState>,
) -> ApiResult<Json<Observation>> {
    let p: Option<ProbeRow> = sqlx::query_as("SELECT * FROM probes WHERE id=?")
        .bind(id)
        .fetch_optional(&s.db)
        .await
        .map_err(internal)?;
    Ok(Json(
        execute(
            &s,
            &p.ok_or(ApiError(StatusCode::NOT_FOUND, "Probe not found".into()))?,
        )
        .await,
    ))
}

#[derive(Deserialize)]
pub struct HistoryQuery {
    probe_id: Option<String>,
}
pub async fn history(
    Query(q): Query<HistoryQuery>,
    State(s): State<AppState>,
) -> ApiResult<Json<Vec<Observation>>> {
    let rows = if let Some(id) = q.probe_id {
        sqlx::query_as(
            "SELECT * FROM observations WHERE probe_id=? ORDER BY started_at DESC LIMIT 500",
        )
        .bind(id)
        .fetch_all(&s.db)
        .await
    } else {
        sqlx::query_as("SELECT * FROM observations ORDER BY started_at DESC LIMIT 500")
            .fetch_all(&s.db)
            .await
    }
    .map_err(internal)?;
    Ok(Json(rows))
}

pub async fn export(State(s): State<AppState>) -> ApiResult<impl IntoResponse> {
    let rows: Vec<ExportRow> = sqlx::query_as("SELECT p.name,p.provider,p.model,o.started_at,o.latency_ms,o.http_status,o.outcome,o.error_class,o.detail,o.input_tokens,o.output_tokens FROM observations o JOIN probes p ON p.id=o.probe_id ORDER BY o.started_at DESC").fetch_all(&s.db).await.map_err(internal)?;
    fn c(v: &str) -> String {
        format!("\"{}\"", v.replace('"', "\"\""))
    }
    let mut csv="probe,provider,model,started_at,latency_ms,http_status,outcome,error_class,detail,input_tokens,output_tokens\n".to_string();
    for r in rows {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{}\n",
            c(&r.name),
            c(&r.provider),
            c(&r.model),
            c(&r.started_at),
            r.latency_ms,
            r.http_status.map(|v| v.to_string()).unwrap_or_default(),
            c(&r.outcome),
            c(r.error_class.as_deref().unwrap_or("")),
            c(r.detail.as_deref().unwrap_or("")),
            r.input_tokens.map(|v| v.to_string()).unwrap_or_default(),
            r.output_tokens.map(|v| v.to_string()).unwrap_or_default()
        ));
    }
    let mut h = HeaderMap::new();
    h.insert(
        header::CONTENT_TYPE,
        "text/csv; charset=utf-8".parse().unwrap(),
    );
    h.insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=capacity-sentinel-observations.csv"
            .parse()
            .unwrap(),
    );
    Ok((h, csv))
}
