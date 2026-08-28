use crate::{
    models::{Observation, ProbeRow, ProbeStats},
    AppState,
};
use chrono::Utc;
use serde_json::{json, Value};
use std::time::{Duration, Instant};
use uuid::Uuid;

pub async fn execute(state: &AppState, probe: &ProbeRow) -> Observation {
    let started_at = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();
    let used_today: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(COALESCE(input_tokens,0)+COALESCE(output_tokens,0)),0) FROM observations WHERE probe_id=? AND started_at >= date('now')"
    ).bind(&probe.id).fetch_one(&state.db).await.unwrap_or(0);
    let estimated_input = ((probe.prompt.chars().count() as i64 + 3) / 4).max(1);
    let estimated_request = estimated_input + probe.max_output_tokens;
    if used_today + estimated_request > probe.daily_token_cap {
        let obs = Observation {
            id,
            probe_id: probe.id.clone(),
            started_at,
            latency_ms: 0,
            http_status: None,
            outcome: "blocked".into(),
            error_class: Some("cost_cap".into()),
            detail: Some(format!(
                "Daily token cap reached ({used_today}/{})",
                probe.daily_token_cap
            )),
            input_tokens: None,
            output_tokens: None,
            invariant_valid: 0,
        };
        save(state, &obs).await;
        evaluate_alerts(state, probe).await;
        return obs;
    }

    let key = match state.secrets.decrypt(&probe.api_key_cipher) {
        Ok(key) => key,
        Err(_) => {
            let obs = Observation {
                id,
                probe_id: probe.id.clone(),
                started_at,
                latency_ms: 0,
                http_status: None,
                outcome: "error".into(),
                error_class: Some("configuration".into()),
                detail: Some("Stored credential could not be decrypted".into()),
                input_tokens: None,
                output_tokens: None,
                invariant_valid: 0,
            };
            save(state, &obs).await;
            return obs;
        }
    };
    let body = json!({
        "model": probe.model,
        "messages": [{"role":"user", "content":probe.prompt}],
        "max_tokens": probe.max_output_tokens,
        "response_format": {"type":"json_object"}
    });
    let begin = Instant::now();
    let result = state
        .client
        .post(&probe.endpoint_url)
        .bearer_auth(key)
        .json(&body)
        .timeout(Duration::from_millis(probe.timeout_ms as u64))
        .send()
        .await;
    let latency_ms = begin.elapsed().as_millis() as i64;

    let obs = match result {
        Err(error) => Observation {
            id,
            probe_id: probe.id.clone(),
            started_at,
            latency_ms,
            http_status: None,
            outcome: "error".into(),
            error_class: Some(
                if error.is_timeout() {
                    "timeout"
                } else {
                    "network"
                }
                .into(),
            ),
            detail: Some(if error.is_timeout() {
                format!("No response within {} ms", probe.timeout_ms)
            } else {
                "Endpoint could not be reached".into()
            }),
            input_tokens: None,
            output_tokens: None,
            invariant_valid: 0,
        },
        Ok(response) => {
            let status = response.status().as_u16() as i64;
            let value: Value = response.json().await.unwrap_or(Value::Null);
            if !(200..300).contains(&status) {
                let class = if status == 429 {
                    "capacity"
                } else if status >= 500 {
                    "upstream"
                } else {
                    "http"
                };
                Observation {
                    id,
                    probe_id: probe.id.clone(),
                    started_at,
                    latency_ms,
                    http_status: Some(status),
                    outcome: "error".into(),
                    error_class: Some(class.into()),
                    detail: Some(format!("Endpoint returned HTTP {status}")),
                    input_tokens: None,
                    output_tokens: None,
                    invariant_valid: 0,
                }
            } else {
                let input_tokens = value
                    .pointer("/usage/prompt_tokens")
                    .and_then(Value::as_i64)
                    .or(Some(estimated_input));
                let output_tokens = value
                    .pointer("/usage/completion_tokens")
                    .and_then(Value::as_i64)
                    .or(Some(probe.max_output_tokens));
                let content = value.pointer("/choices/0/message/content");
                let parsed = match content {
                    Some(Value::String(s)) => parse_json_content(s),
                    Some(other) if other.is_object() => Some(other.clone()),
                    _ => None,
                };
                match parsed {
                    None => Observation {
                        id,
                        probe_id: probe.id.clone(),
                        started_at,
                        latency_ms,
                        http_status: Some(status),
                        outcome: "error".into(),
                        error_class: Some("invalid_json".into()),
                        detail: Some("Response content was not valid JSON".into()),
                        input_tokens,
                        output_tokens,
                        invariant_valid: 0,
                    },
                    Some(doc) => {
                        let missing: Vec<String> = probe
                            .fields()
                            .into_iter()
                            .filter(|p| !has_path(&doc, p))
                            .collect();
                        if missing.is_empty() {
                            Observation {
                                id,
                                probe_id: probe.id.clone(),
                                started_at,
                                latency_ms,
                                http_status: Some(status),
                                outcome: "healthy".into(),
                                error_class: None,
                                detail: Some("All required fields present".into()),
                                input_tokens,
                                output_tokens,
                                invariant_valid: 1,
                            }
                        } else {
                            Observation {
                                id,
                                probe_id: probe.id.clone(),
                                started_at,
                                latency_ms,
                                http_status: Some(status),
                                outcome: "error".into(),
                                error_class: Some("invariant".into()),
                                detail: Some(format!(
                                    "Missing required fields: {}",
                                    missing.join(", ")
                                )),
                                input_tokens,
                                output_tokens,
                                invariant_valid: 0,
                            }
                        }
                    }
                }
            }
        }
    };
    save(state, &obs).await;
    evaluate_alerts(state, probe).await;
    obs
}

fn parse_json_content(raw: &str) -> Option<Value> {
    let trimmed = raw.trim();
    let without_fence = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .and_then(|s| s.strip_suffix("```"))
        .unwrap_or(trimmed)
        .trim();
    serde_json::from_str(without_fence).ok()
}

fn has_path(value: &Value, path: &str) -> bool {
    path.split('.')
        .filter(|p| !p.is_empty())
        .try_fold(value, |node, part| {
            if let Ok(index) = part.parse::<usize>() {
                node.as_array()?.get(index)
            } else {
                node.get(part)
            }
        })
        .is_some()
}

async fn save(state: &AppState, obs: &Observation) {
    let _ = sqlx::query("INSERT INTO observations(id,probe_id,started_at,latency_ms,http_status,outcome,error_class,detail,input_tokens,output_tokens,invariant_valid) VALUES(?,?,?,?,?,?,?,?,?,?,?)")
        .bind(&obs.id).bind(&obs.probe_id).bind(&obs.started_at).bind(obs.latency_ms).bind(obs.http_status)
        .bind(&obs.outcome).bind(&obs.error_class).bind(&obs.detail).bind(obs.input_tokens).bind(obs.output_tokens)
        .bind(obs.invariant_valid).execute(&state.db).await;
}

pub fn calculate_stats(rows: &[Observation]) -> ProbeStats {
    if rows.is_empty() {
        return ProbeStats::default();
    }
    let healthy = rows.iter().filter(|o| o.outcome == "healthy").count();
    let mut latencies: Vec<i64> = rows
        .iter()
        .filter(|o| o.http_status.is_some())
        .map(|o| o.latency_ms)
        .collect();
    latencies.sort_unstable();
    let idx = if latencies.is_empty() {
        0
    } else {
        ((latencies.len() as f64 * 0.95).ceil() as usize).saturating_sub(1)
    };
    ProbeStats {
        sample_count: rows.len(),
        availability_percent: healthy as f64 / rows.len() as f64 * 100.0,
        p95_latency_ms: latencies.get(idx).copied().unwrap_or(0),
        consecutive_failures: rows.iter().take_while(|o| o.outcome != "healthy").count(),
        tokens_today: rows
            .iter()
            .filter(|o| o.started_at.get(..10) == Some(&Utc::now().to_rfc3339()[..10]))
            .map(|o| o.input_tokens.unwrap_or(0) + o.output_tokens.unwrap_or(0))
            .sum(),
    }
}

async fn evaluate_alerts(state: &AppState, probe: &ProbeRow) {
    let rows: Vec<Observation> = sqlx::query_as(
        "SELECT * FROM observations WHERE probe_id=? ORDER BY started_at DESC LIMIT 20",
    )
    .bind(&probe.id)
    .fetch_all(&state.db)
    .await
    .unwrap_or_default();
    let stats = calculate_stats(&rows);
    let last = rows.first();
    let failure_kind = last
        .and_then(|o| o.error_class.as_deref())
        .unwrap_or("availability");
    let failure = stats.consecutive_failures >= 2;
    set_alert(
        state,
        &probe.id,
        "availability",
        failure,
        &format!(
            "{} consecutive failures on {}/{} ({})",
            stats.consecutive_failures, probe.provider, probe.model, failure_kind
        ),
    )
    .await;
    let latency = rows.len() >= 3 && stats.p95_latency_ms > probe.latency_slo_ms;
    set_alert(
        state,
        &probe.id,
        "latency",
        latency,
        &format!(
            "p95 latency is {} ms; objective is {} ms",
            stats.p95_latency_ms, probe.latency_slo_ms
        ),
    )
    .await;
    let shape = stats.consecutive_failures >= 2
        && last
            .and_then(|o| o.error_class.as_deref())
            .map(|c| c == "invariant" || c == "invalid_json")
            .unwrap_or(false);
    set_alert(
        state,
        &probe.id,
        "output_shape",
        shape,
        "Structured output failed validation twice in succession",
    )
    .await;
}

async fn set_alert(state: &AppState, probe_id: &str, kind: &str, active: bool, message: &str) {
    let open: Option<String> = sqlx::query_scalar(
        "SELECT id FROM alerts WHERE probe_id=? AND kind=? AND status='open' LIMIT 1",
    )
    .bind(probe_id)
    .bind(kind)
    .fetch_optional(&state.db)
    .await
    .unwrap_or(None);
    match (active, open) {
        (true, None) => {
            let _=sqlx::query("INSERT INTO alerts(id,probe_id,kind,status,message,opened_at) VALUES(?,?,?,'open',?,?)")
            .bind(Uuid::new_v4().to_string()).bind(probe_id).bind(kind).bind(message).bind(Utc::now().to_rfc3339()).execute(&state.db).await;
        }
        (false, Some(id)) => {
            let _ = sqlx::query("UPDATE alerts SET status='resolved', resolved_at=? WHERE id=?")
                .bind(Utc::now().to_rfc3339())
                .bind(id)
                .execute(&state.db)
                .await;
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_nested_fields_and_fenced_json() {
        let v = parse_json_content("```json\n{\"result\":{\"label\":\"ok\"}}\n```").unwrap();
        assert!(has_path(&v, "result.label"));
        assert!(!has_path(&v, "result.score"));
    }
    #[test]
    fn computes_p95_and_streak() {
        let rows = (0..4)
            .map(|i| Observation {
                id: i.to_string(),
                probe_id: "p".into(),
                started_at: Utc::now().to_rfc3339(),
                latency_ms: (i + 1) * 100,
                http_status: Some(200),
                outcome: if i < 2 { "error" } else { "healthy" }.into(),
                error_class: None,
                detail: None,
                input_tokens: Some(1),
                output_tokens: Some(2),
                invariant_valid: 0,
            })
            .collect::<Vec<_>>();
        let s = calculate_stats(&rows);
        assert_eq!(s.consecutive_failures, 2);
        assert_eq!(s.p95_latency_ms, 400);
        assert_eq!(s.availability_percent, 50.0);
    }
}
