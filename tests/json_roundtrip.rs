use greentic_state::{StateKey, StatePath, StateStore, TenantCtx, inmemory::InMemoryStateStore};
use greentic_types::{EnvId, TenantId};
use proptest::prelude::*;
use serde_json::{Map, Number, Value, json};

fn ctx() -> TenantCtx {
    TenantCtx::new(
        EnvId::try_from("dev").expect("valid env id"),
        TenantId::try_from("tenant").expect("valid tenant id"),
    )
}

#[test]
fn in_memory_roundtrip() {
    let store = InMemoryStateStore::new();
    let ctx = ctx();
    let prefix = "flow/roundtrip";
    let key = StateKey::new("node/a");

    let doc = json!({"a": [1, 2, 3], "status": "ready"});
    store
        .set_json(&ctx, prefix, &key, None, &doc, None)
        .expect("set");

    let loaded = store
        .get_json(&ctx, prefix, &key, None)
        .expect("get")
        .expect("value");
    assert_eq!(loaded, doc);

    let path = StatePath::from_pointer("/a/1");
    let second = store
        .get_json(&ctx, prefix, &key, Some(&path))
        .expect("get")
        .expect("value");
    assert_eq!(second, json!(2));

    store
        .set_json(&ctx, prefix, &key, Some(&path), &json!(42), None)
        .expect("path set");
    let updated = store
        .get_json(&ctx, prefix, &key, None)
        .expect("get")
        .expect("value");
    assert_eq!(updated, json!({"a": [1, 42, 3], "status": "ready"}));

    let replacement = json!({"a": [9, 8], "status": "replaced"});
    store
        .set_json(&ctx, prefix, &key, None, &replacement, None)
        .expect("replace");
    let replaced = store
        .get_json(&ctx, prefix, &key, None)
        .expect("get")
        .expect("value");
    assert_eq!(replaced, replacement);
}

fn json_strategy() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(|v| Value::Number(Number::from(v))),
        any::<String>().prop_map(Value::String),
    ];

    leaf.prop_recursive(4, 64, 8, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..4).prop_map(Value::Array),
            prop::collection::btree_map(any::<String>(), inner, 0..4).prop_map(|map| {
                let mut object = Map::new();
                for (k, v) in map {
                    object.insert(k, v);
                }
                Value::Object(object)
            }),
        ]
    })
}

proptest! {
    #[test]
    fn property_roundtrip(value in json_strategy()) {
        let store = InMemoryStateStore::new();
        let ctx = ctx();
        let prefix = "flow/prop";
        let key = StateKey::new("node/b");
        store.set_json(&ctx, prefix, &key, None, &value, None).expect("set");
        let loaded = store.get_json(&ctx, prefix, &key, None).expect("get");
        prop_assert_eq!(loaded, Some(value.clone()));
    }
}

#[cfg(feature = "redis")]
#[test]
fn redis_roundtrip_when_available() {
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
    let prefix = "flow/redis-roundtrip";
    let key = StateKey::new("node");
    let doc = json!({"hello": "redis", "nums": [1, 2, 3]});

    store
        .set_json(&ctx, prefix, &key, None, &doc, Some(60))
        .expect("set");

    let loaded = store
        .get_json(&ctx, prefix, &key, None)
        .expect("get")
        .expect("value");
    assert_eq!(loaded, doc);
}
