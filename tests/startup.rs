use std::{
    net::TcpListener,
    process::{Child, Command, Stdio},
    thread,
    time::Duration,
};

// @claim:local-persistence-health
#[test]
fn claim_local_persistence_health() {
    let temp = tempfile::tempdir().unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_model-capacity-sentinel"))
        .env_clear()
        .env("PORT", "0")
        .current_dir(temp.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    thread::sleep(Duration::from_secs(2));
    child.kill().unwrap();
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let startup = stdout
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .find(|entry| entry["fields"]["event"] == "startup_configuration")
        .expect("startup configuration record should be visible at the default log level");
    let fields = &startup["fields"];

    assert_eq!(fields["master_key_source"], "generated");
    assert_eq!(fields["access_token_source"], "generated");
    assert_eq!(fields["data_dir_source"], "default");
    assert_eq!(fields["static_dir_source"], "default");
    assert_eq!(fields["port_source"], "supplied");
    assert!(fields.get("access_token").is_none());
    assert!(temp.path().join("data/master.key").exists());
    assert!(temp.path().join("data/access.token").exists());
}

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

fn start_runner(data_dir: &std::path::Path, port: u16) -> Child {
    Command::new(env!("CARGO_BIN_EXE_model-capacity-sentinel"))
        .env_clear()
        .env("PORT", port.to_string())
        .env("DATA_DIR", data_dir)
        .env(
            "SENTINEL_ACCESS_TOKEN",
            "test-access-token-that-is-long-enough",
        )
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

async fn wait_for_health(port: u16) {
    let client = reqwest::Client::new();
    let url = format!("http://127.0.0.1:{port}/health");
    for _ in 0..80 {
        if let Ok(response) = client.get(&url).send().await {
            if response.status().is_success() {
                return;
            }
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("runner did not open its HTTP port");
}

// @claim:durable-project-state
#[tokio::test]
async fn claim_durable_project_state_survives_a_runner_restart() {
    let temp = tempfile::tempdir().unwrap();
    let data_dir = temp.path().join("durable-data");
    let port = free_port();
    let mut first = start_runner(&data_dir, port);
    wait_for_health(port).await;
    let client = reqwest::Client::new();
    let base = format!("http://127.0.0.1:{port}");
    let created = client
        .post(format!("{base}/api/probes"))
        .bearer_auth("test-access-token-that-is-long-enough")
        .json(&serde_json::json!({
            "name": "Persistent capacity canary",
            "provider": "Test provider",
            "endpoint_url": "https://1.1.1.1/chat",
            "model": "test-model",
            "api_key": "test-key",
            "prompt": "Synthetic status only",
            "required_fields": ["status"],
            "interval_minutes": 5,
            "timeout_ms": 1000,
            "latency_slo_ms": 1000,
            "availability_slo_percent": 99,
            "max_output_tokens": 32,
            "daily_token_cap": 1000,
            "enabled": false
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(created.status(), reqwest::StatusCode::CREATED);
    first.kill().unwrap();
    first.wait().unwrap();

    let restarted_port = free_port();
    let mut restarted = start_runner(&data_dir, restarted_port);
    wait_for_health(restarted_port).await;
    let summary: serde_json::Value = client
        .get(format!("http://127.0.0.1:{restarted_port}/api/summary"))
        .bearer_auth("test-access-token-that-is-long-enough")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(summary["probes"][0]["name"], "Persistent capacity canary");
    assert!(data_dir.join("sentinel.db").exists());
    restarted.kill().unwrap();
    restarted.wait().unwrap();
}

// @claim:nonroot-runner
#[cfg(unix)]
#[tokio::test]
async fn claim_nonroot_runner_starts_with_a_writable_data_directory() {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    let temp = tempfile::tempdir().unwrap();
    std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
    let data_dir = temp.path().join("data");
    std::fs::create_dir(&data_dir).unwrap();
    std::fs::set_permissions(&data_dir, std::fs::Permissions::from_mode(0o777)).unwrap();
    let port = free_port();
    let current_uid: u32 = String::from_utf8(Command::new("id").arg("-u").output().unwrap().stdout)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    let expected_uid = if current_uid == 0 { 65532 } else { current_uid };
    assert_ne!(
        expected_uid, 0,
        "the claim must exercise a non-root process"
    );
    let mut command = if current_uid == 0 {
        let mut command = Command::new("setpriv");
        command
            .args(["--reuid=65532", "--regid=65532", "--clear-groups"])
            .arg(env!("CARGO_BIN_EXE_model-capacity-sentinel"));
        command
    } else {
        Command::new(env!("CARGO_BIN_EXE_model-capacity-sentinel"))
    };
    let mut child = command
        .env_clear()
        .env("PORT", port.to_string())
        .env("DATA_DIR", &data_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    wait_for_health(port).await;
    assert!(data_dir.join("sentinel.db").exists());
    assert!(data_dir.join("master.key").exists());
    assert_eq!(
        std::fs::metadata(data_dir.join("sentinel.db"))
            .unwrap()
            .uid(),
        expected_uid,
        "the running non-root user must create the durable database"
    );
    child.kill().unwrap();
    child.wait().unwrap();
}
