use greentic_state::{inmemory::InMemoryStateStore, StateKey, StatePath, StateStore, TenantCtx};
use greentic_types::{EnvId, TenantId};
use serde_json::json;
use uuid::Uuid;

fn main() -> greentic_types::GResult<()> {
    let ctx = TenantCtx::new(EnvId::from("dev"), TenantId::from("example-tenant"));
    let prefix = format!("flow/{}", Uuid::new_v4());
    let key = StateKey::new("node/state");

    let store = InMemoryStateStore::new();
    store.set_json(
        &ctx,
        &prefix,
        &key,
        None,
        &json!({"status": "ready", "attempts": 0}),
        Some(60),
    )?;

    println!(
        "whole document: {}",
        store.get_json(&ctx, &prefix, &key, None)?.unwrap()
    );

    let path = StatePath::from_pointer("/attempts");
    store.set_json(&ctx, &prefix, &key, Some(&path), &json!(1), None)?;
    println!(
        "attempts: {}",
        store.get_json(&ctx, &prefix, &key, Some(&path))?.unwrap()
    );

    #[cfg(feature = "redis")]
    {
        use greentic_state::redis_store::RedisStateStore;
        if let Ok(url) = std::env::var("REDIS_URL") {
            let redis_store = RedisStateStore::from_url(&url)?;
            redis_store.set_json(
                &ctx,
                &prefix,
                &key,
                None,
                &json!({"status": "ready", "backend": "redis"}),
                Some(60),
            )?;
            println!(
                "redis value: {}",
                redis_store.get_json(&ctx, &prefix, &key, None)?.unwrap()
            );
        }
    }

    Ok(())
}
