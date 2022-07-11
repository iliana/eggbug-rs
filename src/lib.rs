#![deny(elided_lifetimes_in_paths)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]

mod attachment;
mod client;
mod error;
mod post;
mod session;

pub use crate::attachment::{Attachment, AttachmentId};
pub use crate::client::Client;
pub use crate::error::Error;
pub use crate::post::{Post, PostId};
pub use crate::session::Session;
