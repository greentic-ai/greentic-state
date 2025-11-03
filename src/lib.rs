#![forbid(unsafe_code)]
#![warn(clippy::unwrap_used, clippy::expect_used)]

//! Multi-tenant JSON state store primitives for Greentic runtimes.

pub mod error;
pub mod inmemory;
pub mod key;
#[cfg(feature = "redis")]
pub mod redis_store;
pub mod store;
pub mod util;

pub use crate::key::{FqnKey, fqn, fqn_prefix};
pub use crate::store::StateStore;
pub use greentic_types::{StateKey, StatePath, TenantCtx};
