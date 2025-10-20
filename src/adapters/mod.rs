/// Adapters layer - Interface adapters that implement repository interfaces
/// Connects domain/use cases to external frameworks and libraries

pub mod repositories;
pub mod platforms;

pub use repositories::*;
