// OutputMode enum, context propagation
//
// The primary output mode resolution logic lives in output/mod.rs:
//   - OutputMode enum
//   - resolve_mode()
//   - JsonTransform struct
//
// This module re-exports those types for architectural clarity.

pub use super::OutputMode;
pub use super::resolve_mode;
pub use super::resolve_mode_full;
pub use super::JsonTransform;
