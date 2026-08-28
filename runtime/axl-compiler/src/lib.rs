//! AXL 4 experimental compiler.
//!
//! The only supported pipeline is readable AXL -> Semantic Graph IR -> Packed IR
//! -> target contracts. Earlier CRM-specific compilers have been removed.

pub mod next;

pub use next::diagnostic::{CheckReport, Diagnostic};
pub use next::{Compilation, compile_file, compile_source, compile_source_at};
