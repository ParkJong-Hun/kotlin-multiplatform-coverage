/// Domain layer - Core business entities and repository interfaces
/// No dependencies on outer layers

pub mod entities;
pub mod repositories;

pub use entities::*;
pub use repositories::*;
