//! eggbug-rs is a bot library for [cohost.org](https://cohost.org/rc/welcome), providing an
//! interface to create, edit, and delete posts.
//!
//! ```no_run
//! use eggbug::{Post, Session};
//!
//! # async fn f() -> Result<(), Box<dyn std::error::Error>> {
//! // Log in
//! let session = Session::login("eggbug@website.invalid", "hunter2").await?;
//!
//! // Describe a post
//! let mut post = Post {
//!     headline: "hello from eggbug-rs!".into(),
//!     markdown: "wow it's like a website in here".into(),
//!     ..Default::default()
//! };
//!
//! // Create the post on the eggbug page
//! let id = session.create_post("eggbug", &mut post).await?;
//!
//! // Oh wait we want to make that a link
//! post.markdown = "wow it's [like a website in here](https://cohost.org/hthrflwrs/post/25147-empty)".into();
//! session.edit_post("eggbug", id, &mut post).await?;
//!
//! // Good job!
//! # Ok(())
//! # }
//! ```

#![deny(elided_lifetimes_in_paths)]
#![warn(clippy::pedantic, missing_docs)]
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
