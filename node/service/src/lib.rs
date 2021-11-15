//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

use sc_service::ChainSpec;

pub mod chain_spec;
pub mod impls;

#[cfg(feature = "with-cygnus-runtime")]
pub use cygnus_runtime;
#[cfg(feature = "with-phoenix-runtime")]
pub use phoenix_runtime;
#[cfg(feature = "with-ulas-runtime")]
pub use ulas_runtime;

pub trait IdentifyVariant {
    fn is_ulas(&self) -> bool;
    fn is_cygnus(&self) -> bool;
    fn is_phoenix(&self) -> bool;
}

impl IdentifyVariant for Box<dyn ChainSpec> {
    fn is_ulas(&self) -> bool {
        self.id().starts_with("ulas")
    }

    fn is_cygnus(&self) -> bool {
        self.id().starts_with("cygnus")
    }

    fn is_phoenix(&self) -> bool {
        self.id().starts_with("phoenix")
    }
}

pub const ULAS_RUNTIME_NOT_AVAILABLE: &str = "Ulas runtime is not available. Please compile the node with `--features with-ulas-runtime` to enable it.";
pub const CYGNUS_RUNTIME_NOT_AVAILABLE: &str = "Cygnus runtime is not available. Please compile the node with `--features with-cygnus-runtime` to enable it.";
pub const PHOENIX_RUNTIME_NOT_AVAILABLE: &str = "Phoenix runtime is not available. Please compile the node with `--features with-phoenix-runtime` to enable it.";
