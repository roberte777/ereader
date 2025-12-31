//! Database queries organized by entity type.

pub mod annotations;
pub mod books;
pub mod collections;
pub mod covers;
pub mod devices;
pub mod reading_states;
pub mod tasks;
pub mod users;

pub use annotations::AnnotationQueries;
pub use books::{BookQueries, BookSortOptions, BookFilterOptions};
pub use collections::CollectionQueries;
pub use covers::CoverQueries;
pub use devices::DeviceQueries;
pub use reading_states::ReadingStateQueries;
pub use tasks::TaskQueries;
pub use users::UserQueries;
