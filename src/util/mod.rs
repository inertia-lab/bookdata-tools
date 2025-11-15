//! Various utility modules.
mod accum;
pub mod logging;
pub mod process;
pub mod serde_string;
pub mod timing;
pub mod unicode;

pub use accum::StringAccumulator;
pub use timing::Timer;
