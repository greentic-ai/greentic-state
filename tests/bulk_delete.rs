use greentic_state::{StateKey, StateStore, TenantCtx, inmemory::InMemoryStateStore};
use greentic_types::{EnvId, TenantId};
use serde_json::json;
use uuid::Uuid;

fn ctx() -> TenantCtx {
    TenantCtx::new(
        EnvId::try_from("dev").expect("valid env id"),
        TenantId::try_from("tenant").expect("valid tenant id"),
    )
}

#[test]
fn in_memory_bulk_delete() {
    let store = InMemoryStateStore::new();
    let ctx = ctx();
    let prefix = "flow/delete";
    let key_a = StateKey::new("node/a");
    let key_b = StateKey::new("node/b");
    let other_prefix = "flow/other";

    store
        .set_json(&ctx, prefix, &key_a, None, &json!({"a": 1}), None)
        .expect("set a");
    store
        .set_json(&ctx, prefix, &key_b, None, &json!({"b": 2}), None)
        .expect("set b");
    store
        .set_json(
            &ctx,
            other_prefix,
            &StateKey::new("node/c"),
            None,
            &json!({"c": 3}),
            None,
        )
        .expect("set other");

    let removed = store.del_prefix(&ctx, prefix).expect("delete prefix");
    assert_eq!(removed, 2);

    assert!(
        store
            .get_json(&ctx, prefix, &key_a, None)
            .expect("get a")
            .is_none()
    );
    assert!(
        store
            .get_json(&ctx, prefix, &key_b, None)
            .expect("get b")
            .is_none()
    );
    assert!(
        store
            .get_json(&ctx, other_prefix, &StateKey::new("node/c"), None)
            .expect("get c")
            .is_some()
    );
}

#[cfg(feature = "redis")]
#[test]
fn redis_bulk_delete_when_available() {
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
    let prefix = format!("flow/delete-{}", Uuid::new_v4());
    let other_prefix = format!("flow/delete-other-{}", Uuid::new_v4());
    let key_a = StateKey::new("node/a");
    let key_b = StateKey::new("node/b");

    store
        .set_json(&ctx, &prefix, &key_a, None, &json!({"redis": 1}), Some(600))
        .expect("set redis a");
    store
        .set_json(&ctx, &prefix, &key_b, None, &json!({"redis": 2}), Some(600))
        .expect("set redis b");
    store
        .set_json(
            &ctx,
            &other_prefix,
            &StateKey::new("node/c"),
            None,
            &json!({"redis": 3}),
            Some(600),
        )
        .expect("set redis other");

    let removed = store.del_prefix(&ctx, &prefix).expect("redis bulk delete");
    assert_eq!(removed, 2);

    assert!(
        store
            .get_json(&ctx, &prefix, &key_a, None)
            .expect("get redis a")
            .is_none()
    );
    assert!(
        store
            .get_json(&ctx, &prefix, &key_b, None)
            .expect("get redis b")
            .is_none()
    );
    assert!(
        store
            .get_json(&ctx, &other_prefix, &StateKey::new("node/c"), None)
            .expect("get redis c")
            .is_some()
    );
}
