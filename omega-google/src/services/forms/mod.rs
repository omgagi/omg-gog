//! Google Forms service module.
//! Provides types and URL builders for the Forms API.

pub mod types;
#[allow(clippy::module_inception)]
pub mod forms;
pub mod responses;

/// Google Forms API v1 base URL.
pub const FORMS_BASE_URL: &str = "https://forms.googleapis.com/v1";

// Re-export commonly used functions from submodules for convenience.
pub use forms::{build_form_get_url, build_form_create_url, build_form_create_body};
pub use responses::{build_responses_list_url, build_response_get_url};
