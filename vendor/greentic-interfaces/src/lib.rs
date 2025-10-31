//! Minimal ABI placeholder used by greentic-state.
#![forbid(unsafe_code)]

/// Marker module ensuring the crate links successfully even without bindings.
pub mod abi {
    /// Placeholder type exported to satisfy downstream compilation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Dummy;
}
