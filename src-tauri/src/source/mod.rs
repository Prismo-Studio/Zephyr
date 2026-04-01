//! Multi-source mod abstraction layer for Zephyr.
//!
//! This module defines the `ModSource` trait that all mod sources must implement,
//! along with common types used across sources. This is what makes Zephyr different
//! Instead of hardcoding Thunderstore, we abstract over multiple sources.

pub mod commands;
pub mod registry;
pub mod thunderstore_adapter;
pub mod types;

pub use registry::SourceRegistry;
pub use types::*;
