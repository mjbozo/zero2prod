//! src/routes/admin/newsletters/mod.rs

mod get;
mod post;

pub use get::publish_newsletter_form;
pub use post::publish_newsletter;
