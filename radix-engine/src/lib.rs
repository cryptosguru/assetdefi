#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(any(feature = "std", feature = "alloc")))]
compile_error!("Either feature `std` or `alloc` must be enabled for this crate.");
#[cfg(all(feature = "std", feature = "alloc"))]
compile_error!("Feature `std` and `alloc` can't be enabled at the same time.");

/// In-memory engine.
pub mod engine;
/// Execution and runtime.
pub mod execution;
/// Ledger abstraction.
pub mod ledger;
/// Kernel data models.
pub mod model;
