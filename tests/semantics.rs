use greentic_state::{StateKey, StateStore, TenantCtx, inmemory::InMemoryStateStore};
use greentic_types::{EnvId, TenantId};
use serde_json::json;
use uuid::Uuid;

fn ctx(tenant: &str) -> TenantCtx {
    TenantCtx::new(
        EnvId::try_from("dev").expect("valid env id"),
        TenantId::try_from(tenant).expect("valid tenant id"),
    )
}

#[test]
fn in_memory_delete_removes_entry() {
    let store = InMemoryStateStore::new();
    let ctx = ctx("tenant-a");
    let prefix = "flow/delete";
    let key = StateKey::new("node/a");

    store
        .set_json(&ctx, prefix, &key, None, &json!({"a": 1}), None)
        .expect("set");

    let removed = store.del(&ctx, prefix, &key).expect("delete");
    assert!(removed, "expected delete to return true for existing key");

    let value = store.get_json(&ctx, prefix, &key, None).expect("get");
    assert!(value.is_none(), "expected deleted key to be gone");

    let removed_again = store.del(&ctx, prefix, &key).expect("delete");
    assert!(
        !removed_again,
        "expected delete to return false for missing key"
    );
}

#[test]
fn in_memory_prefix_delete_is_tenant_scoped() {
    let store = InMemoryStateStore::new();
    let ctx_a = ctx("tenant-a");
    let ctx_b = ctx("tenant-b");
    let prefix = "flow/shared";
    let key = StateKey::new("node/a");

    store
        .set_json(&ctx_a, prefix, &key, None, &json!({"a": 1}), None)
        .expect("set a");
    store
        .set_json(&ctx_b, prefix, &key, None, &json!({"b": 2}), None)
        .expect("set b");

    let removed = store.del_prefix(&ctx_a, prefix).expect("delete prefix");
    assert_eq!(removed, 1, "expected only tenant-a entries removed");

    let still_there = store.get_json(&ctx_b, prefix, &key, None).expect("get");
    assert!(still_there.is_some(), "expected tenant-b entry to remain");
}

#[cfg(feature = "redis")]
#[test]
fn redis_prefix_delete_is_tenant_scoped() {
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

    let ctx_a = ctx("tenant-a");
    let ctx_b = ctx("tenant-b");
    let prefix = format!("flow/shared-{}", Uuid::new_v4());
    let key = StateKey::new("node/a");

    store
        .set_json(&ctx_a, &prefix, &key, None, &json!({"a": 1}), None)
        .expect("set a");
    store
        .set_json(&ctx_b, &prefix, &key, None, &json!({"b": 2}), None)
        .expect("set b");

    let removed = store.del_prefix(&ctx_a, &prefix).expect("delete prefix");
    assert_eq!(removed, 1, "expected only tenant-a entries removed");

    let still_there = store.get_json(&ctx_b, &prefix, &key, None).expect("get");
    assert!(still_there.is_some(), "expected tenant-b entry to remain");
}
