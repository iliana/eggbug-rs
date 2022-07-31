/// Errors that might occur when using the library.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Attempted to create or edit a post with no headline, attachments, or markdown content.
    #[error("post is empty (no headline, attachments, or markdown)")]
    EmptyPost,

    /// Attempted to create or edit a post with an [`Attachment`][`crate::Attachment`] marked as
    /// failed.
    #[error("attempted to use post with failed attachment")]
    FailedAttachment,

    /// An error while decoding a Base64 string.
    #[error("base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),

    /// An I/O error.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),

    /// An HTTP client error (including status codes indicating failure).
    #[error("request error: {0}")]
    Request(#[from] reqwest::Error),
}
