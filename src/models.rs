use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct ProbeRow {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub endpoint_url: String,
    pub model: String,
    pub api_key_cipher: String,
    pub prompt: String,
    pub required_fields: String,
    pub interval_minutes: i64,
    pub timeout_ms: i64,
    pub latency_slo_ms: i64,
    pub availability_slo_percent: f64,
    pub max_output_tokens: i64,
    pub daily_token_cap: i64,
    pub enabled: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ProbeView {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub endpoint_url: String,
    pub model: String,
    pub has_api_key: bool,
    pub prompt: String,
    pub required_fields: Vec<String>,
    pub interval_minutes: i64,
    pub timeout_ms: i64,
    pub latency_slo_ms: i64,
    pub availability_slo_percent: f64,
    pub max_output_tokens: i64,
    pub daily_token_cap: i64,
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
    pub last_observation: Option<Observation>,
    pub stats: ProbeStats,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProbeInput {
    pub name: String,
    pub provider: String,
    pub endpoint_url: String,
    pub model: String,
    #[serde(default)]
    pub api_key: String,
    pub prompt: String,
    #[serde(default)]
    pub required_fields: Vec<String>,
    #[serde(default = "default_interval")]
    pub interval_minutes: i64,
    #[serde(default = "default_timeout")]
    pub timeout_ms: i64,
    #[serde(default = "default_latency")]
    pub latency_slo_ms: i64,
    #[serde(default = "default_availability")]
    pub availability_slo_percent: f64,
    #[serde(default = "default_output")]
    pub max_output_tokens: i64,
    #[serde(default = "default_cap")]
    pub daily_token_cap: i64,
    #[serde(default = "default_true")]
    pub enabled: bool,
}
fn default_interval() -> i64 {
    5
}
fn default_timeout() -> i64 {
    15000
}
fn default_latency() -> i64 {
    3000
}
fn default_availability() -> f64 {
    99.0
}
fn default_output() -> i64 {
    128
}
fn default_cap() -> i64 {
    5000
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Observation {
    pub id: String,
    pub probe_id: String,
    pub started_at: String,
    pub latency_ms: i64,
    pub http_status: Option<i64>,
    pub outcome: String,
    pub error_class: Option<String>,
    pub detail: Option<String>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub invariant_valid: i64,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Alert {
    pub id: String,
    pub probe_id: String,
    pub kind: String,
    pub status: String,
    pub message: String,
    pub opened_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct ProbeStats {
    pub sample_count: usize,
    pub availability_percent: f64,
    pub p95_latency_ms: i64,
    pub consecutive_failures: usize,
    pub tokens_today: i64,
}

impl ProbeRow {
    pub fn fields(&self) -> Vec<String> {
        serde_json::from_str(&self.required_fields).unwrap_or_default()
    }
}
