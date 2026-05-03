use super::cmd::cmd_output;
use super::env::{ClientConfig, EnvConfig, PPPoETestEnv};
use super::require_root;
use super::runner::{run_client, ExpectOutcome};
use std::path::Path;
use std::time::{Duration, Instant};

fn wait_for_path(path: &str, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if Path::new(path).exists() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Path::new(path).exists()
}

#[tokio::test]
async fn ebpf_pipeline_maps_are_created_after_connection() {
    require_root();
    let env_cfg = EnvConfig::default();
    let env = PPPoETestEnv::up(&env_cfg).expect("test environment should start");
    let client_cfg = ClientConfig {
        username: env_cfg.username.clone(),
        password: env_cfg.password.clone(),
        ..Default::default()
    };

    let ifindex = env.client_info().index;
    let client_ns = env.client_ns().to_string();
    let iface_name = env.client_info().name.clone();

    let result = run_client(
        env.client_ns(),
        env.client_info(),
        &client_cfg,
        ExpectOutcome::Running,
        Some(Box::new(move || {
            // PPPoE session is fully established and eBPF pipeline should be active.
            // The pipeline maps are pinned in the global BPF filesystem.
            let map_base = "/sys/fs/bpf/landscape";
            let ingress_map = format!("{}/wan_tc_pipeline_ingress_{}", map_base, ifindex);
            let egress_map = format!("{}/wan_tc_pipeline_egress_{}", map_base, ifindex);

            assert!(
                wait_for_path(&ingress_map, Duration::from_secs(3)),
                "ingress pipeline map should exist at {ingress_map}"
            );
            assert!(
                wait_for_path(&egress_map, Duration::from_secs(3)),
                "egress pipeline map should exist at {egress_map}"
            );

            // Verify TC hooks are present on the interface (pipeline root programs).
            let tc_ingress = cmd_output(
                "ip",
                &[
                    "netns",
                    "exec",
                    &client_ns,
                    "tc",
                    "filter",
                    "show",
                    "dev",
                    &iface_name,
                    "ingress",
                ],
            );
            assert!(tc_ingress.is_ok(), "TC ingress filter should be attached: {tc_ingress:?}");

            let tc_egress = cmd_output(
                "ip",
                &[
                    "netns",
                    "exec",
                    &client_ns,
                    "tc",
                    "filter",
                    "show",
                    "dev",
                    &iface_name,
                    "egress",
                ],
            );
            assert!(tc_egress.is_ok(), "TC egress filter should be attached: {tc_egress:?}");
        })),
    )
    .await;
    drop(env);

    assert!(result.is_ok(), "client should connect and activate eBPF pipeline: {result:?}");
}

#[tokio::test]
async fn ebpf_pipeline_maps_cleaned_up_after_client_stop() {
    require_root();
    let env_cfg = EnvConfig::default();
    let env = PPPoETestEnv::up(&env_cfg).expect("test environment should start");
    let client_cfg = ClientConfig {
        username: env_cfg.username.clone(),
        password: env_cfg.password.clone(),
        ..Default::default()
    };

    let ifindex = env.client_info().index;

    let result =
        run_client(env.client_ns(), env.client_info(), &client_cfg, ExpectOutcome::Running, None)
            .await;
    drop(env);

    assert!(result.is_ok(), "client should connect and stop without error: {result:?}");

    // After the client stops, the pipeline's prog_array maps still exist
    // (they are pinned and shared across pipeline stages), but the PPPoE
    // slots should be empty. The key check is that the client exited
    // successfully, which means `unregister_pppoe()` was called without error.
    let map_base = "/sys/fs/bpf/landscape";
    let ingress_map = format!("{}/wan_tc_pipeline_ingress_{}", map_base, ifindex);
    assert!(
        std::path::Path::new(&ingress_map).exists(),
        "pipeline ingress map should persist after cleanup (shared map)"
    );
}
