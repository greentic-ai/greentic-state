# Component State in Greentic Flows

This guide describes how hosts/runners persist flow component state using
`greentic-state`, plus how to access payloads produced by previous nodes in the
same flow.

Components (Wasm guests) do NOT import `greentic-state` directly. Hosts/runners
use it as the backing store for the WIT interface
`greentic:state/store@1.0.0`.

## Naming and scoping

Every state operation is scoped by:
- `TenantCtx` (env + tenant + optional team/user)
- `prefix` (the flow execution namespace)
- `StateKey` (the per-node key)

Pick a stable naming convention for your flow. For example:

- `prefix`: `flow/{flow_id}` or `flow/{flow_id}/{run_id}`
- `StateKey`: `node/{node_id}` or `node/{node_id}/output`

If all nodes in a flow use the same `prefix`, any node can read the state from
any other node by using that node's `StateKey`.

## Add (create) state

Use `set_json` with no `StatePath` to write a whole document. The TTL is
optional; pass `Some(seconds)` to set or refresh expiry, `None` to preserve any
existing TTL, or `Some(0)` to clear an existing TTL.

```rust
use greentic_state::{StateKey, StateStore, TenantCtx};
use serde_json::json;

let key = StateKey::new("node/transform");
store.set_json(&ctx, prefix, &key, None, &json!({
    "payload": {"count": 42},
    "status": "ready"
}), Some(300))?;
```

## Get state

Use `get_json` with no path to read the whole document, or a `StatePath` to read
just a nested value.

```rust
use greentic_state::StatePath;

let key = StateKey::new("node/transform");
let full = store.get_json(&ctx, prefix, &key, None)?;

let path = StatePath::from_pointer("/payload");
let payload_only = store.get_json(&ctx, prefix, &key, Some(&path))?;
```

## Update state

Use `set_json` with a `StatePath` to update a single field inside the JSON
document.

```rust
use greentic_state::StatePath;
use serde_json::json;

let key = StateKey::new("node/transform");
let path = StatePath::from_pointer("/status");
store.set_json(&ctx, prefix, &key, Some(&path), &json!("running"), None)?;
```

## Delete state

Use `del` for a single node, or `del_prefix` to wipe a whole flow namespace.

```rust
let key = StateKey::new("node/transform");
store.del(&ctx, prefix, &key)?;

// End of flow cleanup:
store.del_prefix(&ctx, prefix)?;
```

## Access payload from previous nodes

To read a previous node's output, use the same `prefix` and that node's
`StateKey`. If you store outputs under a known field (e.g. `/payload`), you can
read just that value.

```rust
use greentic_state::{StateKey, StatePath};

let prev_key = StateKey::new("node/fetch-data");
let payload_path = StatePath::from_pointer("/payload");
let prev_payload = store.get_json(&ctx, prefix, &prev_key, Some(&payload_path))?;

// Or read the entire document for that node:
let prev_state = store.get_json(&ctx, prefix, &prev_key, None)?;
```

Notes:
- All keys are tenant-scoped via `TenantCtx`, so cross-tenant reads are blocked
  by design.
- If you pass `None` for `ttl_secs` in `set_json`, existing TTLs are preserved.
