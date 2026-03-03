#[allow(clippy::module_inception)]
pub mod contacts;
pub mod directory;
pub mod types;

/// People API base URL (Contacts uses People API internally).
pub const PEOPLE_API_BASE_URL: &str = "https://people.googleapis.com/v1";
