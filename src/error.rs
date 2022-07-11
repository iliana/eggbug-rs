#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("post is empty (no headline, attachments, or markdown)")]
    EmptyPost,

    #[error("attempted to use post with failed attachment")]
    FailedAttachment,

    #[error("no project specified for post")]
    NoProject,

    #[error("base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),

    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),

    #[error("request error: {0}")]
    Request(#[from] reqwest::Error),
}
