/// Adapters layer - Interface adapters that implement repository interfaces
/// Connects domain/use cases to external frameworks and libraries

pub mod repositories;
pub mod platforms;
pub mod project_detector;
pub mod platform_detector;

pub use repositories::*;
pub use project_detector::{ProjectDetector, DetectedProject, ProjectType};
