use greentic_state::{StateKey, StateStore, TenantCtx, inmemory::InMemoryStateStore};
use greentic_types::{EnvId, TenantId};
use serde_json::json;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

fn ctx() -> TenantCtx {
    TenantCtx::new(EnvId::from("dev"), TenantId::from("tenant"))
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn in_memory_ttl_expires() {
    let store = InMemoryStateStore::new();
    let ctx = ctx();
    let prefix = "flow/ttl-in-memory";
    let key = StateKey::new("node/a");

    store
        .set_json(&ctx, prefix, &key, None, &json!({"ttl": true}), Some(1))
        .expect("set");

    sleep(Duration::from_millis(1_100)).await;

    let value = store.get_json(&ctx, prefix, &key, None).expect("get");
    assert!(value.is_none(), "expected value to expire");
}

#[cfg(feature = "redis")]
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn redis_ttl_expires_when_configured() {
    use greentic_state::redis_store::RedisStateStore;
    use std::env;

    let url = match env::var("REDIS_URL") {
        Ok(url) => url,
        Err(_) => return,
    };
    let store = match RedisStateStore::from_url(&url) {
        Ok(store) => store,
        Err(_) => return,
    };

    let ctx = ctx();
    let prefix = format!("flow/ttl-redis-{}", Uuid::new_v4());
    let key = StateKey::new("node/a");

    store
        .set_json(&ctx, &prefix, &key, None, &json!({"redis": true}), Some(1))
        .expect("set redis ttl");

    sleep(Duration::from_millis(1_100)).await;

    let value = store
        .get_json(&ctx, &prefix, &key, None)
        .expect("get redis TTL");
    assert!(value.is_none(), "expected redis TTL to expire value");
}
