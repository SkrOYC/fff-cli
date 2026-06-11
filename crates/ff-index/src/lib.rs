//! In-memory file index for ff.

pub mod entry;
pub mod scanner;
pub mod store;

pub use entry::FileEntry;
pub use scanner::{ScanOptions, scan};
pub use store::Index;
