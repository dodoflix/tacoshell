//! # Tacoshell Core
//!
//! Core types, traits, and error handling shared across all Tacoshell crates.

pub mod error;
pub mod models;
pub mod traits;

pub use error::{Error, Result};
pub use models::*;
pub use traits::*;

