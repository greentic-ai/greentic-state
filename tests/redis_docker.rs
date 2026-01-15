#[cfg(feature = "redis")]
mod redis_docker {
    use greentic_state::redis_store::RedisStateStore;
    use greentic_state::{StateKey, StateStore, TenantCtx};
    use greentic_types::{EnvId, TenantId};
    use serde_json::json;
    use std::env;
    use std::process::{Command, Stdio};
    use std::thread::sleep;
    use std::time::{Duration, Instant};
    use uuid::Uuid;

    fn ctx() -> TenantCtx {
        TenantCtx::new(
            EnvId::try_from("dev").expect("valid env id"),
            TenantId::try_from("tenant").expect("valid tenant id"),
        )
    }

    fn docker_available() -> bool {
        Command::new("docker")
            .arg("version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    struct RedisContainer {
        name: String,
    }

    impl Drop for RedisContainer {
        fn drop(&mut self) {
            let _ = Command::new("docker")
                .args(["stop", &self.name])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }

    fn start_redis_container() -> Option<(RedisContainer, u16)> {
        let name = format!("greentic-state-test-{}", Uuid::new_v4());

        let status = Command::new("docker")
            .args([
                "run", "-d", "--rm", "--name", &name, "-p", "0:6379", "redis:7",
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .ok()?;

        if !status.success() {
            return None;
        }

        let container = RedisContainer { name };
        let port = resolve_container_port(&container.name)?;
        if !wait_for_redis(&container.name) {
            return None;
        }

        Some((container, port))
    }

    fn resolve_container_port(name: &str) -> Option<u16> {
        let output = Command::new("docker")
            .args(["port", name, "6379/tcp"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let line = stdout.lines().next()?;
        let port_str = line.rsplit(':').next()?;
        port_str.trim().parse::<u16>().ok()
    }

    fn wait_for_redis(name: &str) -> bool {
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(30) {
            let status = Command::new("docker")
                .args(["exec", name, "redis-cli", "ping"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            if matches!(status, Ok(status) if status.success()) {
                return true;
            }
            sleep(Duration::from_millis(200));
        }
        false
    }

    #[test]
    fn redis_integration_via_docker() {
        if env::var("SKIP_DOCKER_REDIS_TEST").is_ok() {
            return;
        }
        let mut _container = None;
        let url = if let Ok(url) = env::var("REDIS_URL") {
            url
        } else {
            if !docker_available() {
                return;
            }
            let Some((container, port)) = start_redis_container() else {
                return;
            };
            _container = Some(container);
            format!("redis://127.0.0.1:{port}")
        };
        let store = RedisStateStore::from_url(&url).expect("connect redis");
        let ctx = ctx();
        let prefix = format!("flow/redis-docker-{}", Uuid::new_v4());
        let key = StateKey::new("node/a");

        store
            .set_json(
                &ctx,
                &prefix,
                &key,
                None,
                &json!({"hello": "world"}),
                Some(60),
            )
            .expect("set");
        let loaded = store
            .get_json(&ctx, &prefix, &key, None)
            .expect("get")
            .expect("value");
        assert_eq!(loaded, json!({"hello": "world"}));

        let removed = store.del(&ctx, &prefix, &key).expect("delete");
        assert!(removed, "expected delete to return true");
        let missing = store.get_json(&ctx, &prefix, &key, None).expect("get");
        assert!(missing.is_none(), "expected deleted key to be gone");

        store
            .set_json(
                &ctx,
                &prefix,
                &StateKey::new("node/b"),
                None,
                &json!({"b": 1}),
                None,
            )
            .expect("set b");
        store
            .set_json(
                &ctx,
                &prefix,
                &StateKey::new("node/c"),
                None,
                &json!({"c": 2}),
                None,
            )
            .expect("set c");
        let removed = store.del_prefix(&ctx, &prefix).expect("delete prefix");
        assert!(removed >= 2, "expected prefix delete to remove entries");
    }
}
