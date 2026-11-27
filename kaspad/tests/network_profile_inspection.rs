use kaspa_consensus_core::config::params::{BlockrateParams, SIMNET_GENESIS};
use serde_json::Value;
use std::{fs, process::Command};

#[test]
fn print_and_reject_network_profiles_without_starting_the_daemon() {
    let directory = tempfile::tempdir().unwrap();
    let appdir = directory.path().join("node");
    let logdir = directory.path().join("logs");
    let overrides = directory.path().join("profile.json");
    let mut command = Command::new(env!("CARGO_BIN_EXE_kaspad"));
    command.args(["--inspect-network-profile", "--simnet", "--nodnsseed"]);
    command.arg(format!("--appdir={}", appdir.display()));
    command.arg(format!("--logdir={}", logdir.display()));
    command.arg(format!("--override-params-file={}", overrides.display()));
    fs::write(
        &overrides,
        serde_json::json!({"blockrate": BlockrateParams::new::<5>(), "skip_proof_of_work": false, "shielded_coinbase": false})
            .to_string(),
    )
    .unwrap();
    let valid = command.output().unwrap();
    assert!(valid.status.success(), "{}", String::from_utf8_lossy(&valid.stderr));
    let profile: Value = serde_json::from_slice(&valid.stdout).unwrap();
    assert_eq!(profile["network_id"], "simnet");
    assert_eq!(profile["configured_genesis_hash"], SIMNET_GENESIS.hash.to_string());
    assert_eq!(profile["target_time_per_block_ms"], 200);
    assert_eq!(profile["skip_proof_of_work"], false);
    assert_eq!(profile["dns_seeding_enabled"], false);
    assert!(!appdir.exists() && !logdir.exists());

    fs::write(&overrides, r#"{"pre_crescendo_target_time_per_block":0}"#).unwrap();
    let invalid = command.output().unwrap();
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("pre_crescendo_target_time_per_block"));
    assert!(!appdir.exists() && !logdir.exists());
}

#[test]
fn inspect_uses_configured_override_unless_the_cli_supplies_one() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let config = root.join("kaspad.toml");
    fs::write(&config, "simnet = true\noverride-params-file = 'file-profile.json'\n").unwrap();
    fs::write(root.join("file-profile.json"), r#"{"skip_proof_of_work":false}"#).unwrap();
    fs::write(root.join("cli-profile.json"), "{}").unwrap();

    let mut command = Command::new(env!("CARGO_BIN_EXE_kaspad"));
    command.current_dir(root).args(["--configfile", "kaspad.toml", "--inspect-network-profile"]);
    let from_config = command.output().unwrap();
    assert!(from_config.status.success(), "{}", String::from_utf8_lossy(&from_config.stderr));
    let profile: Value = serde_json::from_slice(&from_config.stdout).unwrap();
    assert_eq!(profile["network_id"], "simnet");
    assert_eq!(profile["skip_proof_of_work"], false);

    let from_cli = command.arg("--override-params-file=cli-profile.json").output().unwrap();
    assert!(from_cli.status.success(), "{}", String::from_utf8_lossy(&from_cli.stderr));
    let profile: Value = serde_json::from_slice(&from_cli.stdout).unwrap();
    assert_eq!(profile["skip_proof_of_work"], true);
}
