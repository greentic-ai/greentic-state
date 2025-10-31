use greentic_types::{StateKey, TenantCtx};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// Optional JSON pointer-like path into a stored JSON value (e.g., `/a/b/0`).
///
/// The canonical definition of `StatePath` lives in `greentic-types`; this crate re-exports it
/// through [`crate::StatePath`].
pub type StatePath = greentic_types::StatePath;

/// Fully-qualified storage key derived from [`TenantCtx`], a caller-provided prefix, and a [`StateKey`].
///
/// Storage backends MUST only use the string representation provided by [`fqn`].
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FqnKey(pub String);

impl FqnKey {
    /// Returns the key as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for FqnKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl AsRef<str> for FqnKey {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Deterministic FQN composer. Never include secrets in inputs.
pub fn fqn(tenant: &TenantCtx, prefix: &str, key: &StateKey) -> FqnKey {
    let scope = tenant_scope(tenant);
    FqnKey(format!(
        "greentic:state:{scope}:{prefix}:{key}",
        key = key.as_str()
    ))
}

/// Compute the namespaced prefix used for bulk deletion (namespace-level).
pub fn fqn_prefix(tenant: &TenantCtx, prefix: &str) -> String {
    let scope = tenant_scope(tenant);
    format!("greentic:state:{scope}:{prefix}:")
}

fn tenant_scope(tenant: &TenantCtx) -> String {
    let mut segments = vec![tenant.env.as_str(), tenant.tenant_id.as_str()];

    if let Some(team) = tenant.team_id.as_ref().or(tenant.team.as_ref()) {
        segments.push(team.as_ref());
    }

    if let Some(user) = tenant.user_id.as_ref().or(tenant.user.as_ref()) {
        segments.push(user.as_ref());
    }

    segments.join(":")
}

#[cfg(test)]
mod tests {
    use super::*;
    use greentic_types::{EnvId, TeamId, TenantCtx, TenantId, UserId};

    fn ctx() -> TenantCtx {
        TenantCtx::new(EnvId::from("dev"), TenantId::from("tenant"))
            .with_team(Some(TeamId::from("team")))
            .with_user(Some(UserId::from("user")))
    }

    #[test]
    fn fqn_includes_scope() {
        let ctx = ctx();
        let key = StateKey::new("flow/abc");
        let fqn = fqn(&ctx, "global", &key);
        assert_eq!(
            fqn.as_str(),
            "greentic:state:dev:tenant:team:user:global:flow/abc"
        );
    }

    #[test]
    fn prefix_matches_fqn() {
        let ctx = ctx();
        let key = StateKey::new("flow/abc");
        let prefix = fqn_prefix(&ctx, "global");
        assert!(fqn(&ctx, "global", &key).as_str().starts_with(&prefix));
    }
}
