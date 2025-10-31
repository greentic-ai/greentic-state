use crate::key::StatePath;
use greentic_types::{GResult, StateKey, TenantCtx};
use serde_json::Value;

/// JSON state store operations shared across backends.
pub trait StateStore: Send + Sync + 'static {
    /// Get the JSON value for `(tenant, prefix, key)`.
    /// When `path` is provided the returned value corresponds to that JSON Pointer.
    fn get_json(
        &self,
        tenant: &TenantCtx,
        prefix: &str,
        key: &StateKey,
        path: Option<&StatePath>,
    ) -> GResult<Option<Value>>;

    /// Set the JSON value for `(tenant, prefix, key)`.
    /// When `path` is provided the value is upserted at the JSON Pointer location.
    /// Passing `ttl_secs` refreshes the expiry; `None` keeps the existing TTL (if any).
    fn set_json(
        &self,
        tenant: &TenantCtx,
        prefix: &str,
        key: &StateKey,
        path: Option<&StatePath>,
        value: &Value,
        ttl_secs: Option<u32>,
    ) -> GResult<()>;

    /// Delete the entire JSON value at `(tenant, prefix, key)`.
    /// Returns `true` when the key existed.
    fn del(&self, tenant: &TenantCtx, prefix: &str, key: &StateKey) -> GResult<bool>;

    /// Bulk delete all keys under `(tenant, prefix)` — used for flow cleanup, etc.
    /// Returns the number of entries removed.
    fn del_prefix(&self, tenant: &TenantCtx, prefix: &str) -> GResult<u64>;
}
