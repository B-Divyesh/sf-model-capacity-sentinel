use std::{process::{Command, Stdio}, thread, time::Duration};

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
