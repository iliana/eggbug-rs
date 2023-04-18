use crate::{Client, Error, PostId};
use bytes::Bytes;
use derive_more::{Display, From, FromStr, Into};
use reqwest::multipart::{Form, Part};
use reqwest::Body;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// An attachment ID.
#[allow(clippy::module_name_repetitions)]
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Display,
    Eq,
    From,
    FromStr,
    Hash,
    Into,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
#[serde(transparent)]
pub struct AttachmentId(pub Uuid);

/// Describes an attachment.
///
/// Attachments are created in the ["new"][`Attachment::is_new`] state. When part of a
/// [`Post`][`crate::Post`] that is created or edited, the client attempts to upload the
/// attachment. If successful, the attachment becomes ["uploaded"][`Attachment::is_uploaded`]; if
/// not, the attachment becomes ["failed"][`Attachment::is_failed`].
#[derive(Debug)]
pub struct Attachment {
    pub(crate) kind: Inner,

    /// Alt text associated with this attachment.
    pub alt_text: String,
}

#[derive(Debug)]
pub(crate) enum Inner {
    New {
        stream: Body,
        filename: String,
        content_type: String,
        content_length: u64,
        width: Option<u32>,
        height: Option<u32>,
    },
    Uploaded(Finished),
    Failed,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Finished {
    pub(crate) attachment_id: AttachmentId,
    pub(crate) url: String,
}

impl Attachment {
    /// Create an `Attachment` from a buffer.
    ///
    /// # Panics
    ///
    /// Panics if the length of `content` overflows a [`u64`].
    pub fn new(
        content: impl Into<Bytes>,
        filename: String,
        content_type: String,
        width: Option<u32>,
        height: Option<u32>,
    ) -> Attachment {
        let content: Bytes = content.into();
        Attachment {
            kind: Inner::New {
                content_length: content.len().try_into().unwrap(),
                stream: content.into(),
                filename,
                content_type,
                width,
                height,
            },
            alt_text: String::new(),
        }
    }

    /// Create an `Attachment` from a file on disk.
    #[cfg(feature = "fs")]
    pub async fn new_from_file(
        path: impl AsRef<std::path::Path> + Clone,
        content_type: String,
    ) -> Result<Attachment, std::io::Error> {
        use tokio::fs::File;
        use tokio_util::codec::{BytesCodec, FramedRead};

        let filename = path
            .as_ref()
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("file")
            .to_owned();

        let file = File::open(path.clone()).await?;
        let content_length = file.metadata().await?.len();
        let stream = Body::wrap_stream(FramedRead::new(file, BytesCodec::new()));

        let (width, height) = match imagesize::size(path.clone()) {
            Ok(dim) => (Some(dim.width as u32), Some(dim.height as u32)),
            Err(_) => (None, None),
        };

        Ok(Attachment {
            kind: Inner::New {
                stream,
                filename,
                content_type,
                content_length,
                width,
                height,
            },
            alt_text: String::new(),
        })
    }

    /// Sets new alt text in a builder-style function.
    #[must_use]
    pub fn with_alt_text(self, alt_text: String) -> Attachment {
        Attachment {
            kind: self.kind,
            alt_text,
        }
    }

    /// Returns true if the attachment has not yet been uploaded.
    pub fn is_new(&self) -> bool {
        matches!(self.kind, Inner::New { .. })
    }

    /// Returns true if the attachment is uploaded.
    pub fn is_uploaded(&self) -> bool {
        matches!(self.kind, Inner::Uploaded { .. })
    }

    /// Returns true if the attachment failed to upload. Failed attachments cannot be recovered.
    pub fn is_failed(&self) -> bool {
        matches!(self.kind, Inner::Failed)
    }

    /// If the attachment is uploaded, returns the CDN URL.
    pub fn url(&self) -> Option<&str> {
        match &self.kind {
            Inner::Uploaded(Finished { url, .. }) => Some(url),
            _ => None,
        }
    }

    pub(crate) fn id(&self) -> Option<AttachmentId> {
        match self.kind {
            Inner::Uploaded(Finished { attachment_id, .. }) => Some(attachment_id),
            _ => None,
        }
    }

    #[tracing::instrument(skip(client))]
    pub(crate) async fn upload(
        &mut self,
        client: &Client,
        project: &str,
        id: PostId,
    ) -> Result<(), Error> {
        let (stream, filename, content_type, content_length, width, height) =
            match std::mem::replace(&mut self.kind, Inner::Failed) {
                Inner::New {
                    stream,
                    filename,
                    content_type,
                    content_length,
                    width,
                    height,
                } => (
                    stream,
                    filename,
                    content_type,
                    content_length,
                    width,
                    height,
                ),
                Inner::Uploaded(_) => return Ok(()),
                Inner::Failed => return Err(Error::FailedAttachment),
            };

        let response: AttachStartResponse = client
            .post(&format!("project/{}/posts/{}/attach/start", project, id))
            .json(&AttachStartRequest {
                filename: &filename,
                content_type: &content_type,
                content_length,
                width,
                height,
            })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        tracing::info!(attachment_id = %response.attachment_id);

        let mut form = Form::new();
        for (name, value) in response.required_fields {
            form = form.text(name, value);
        }
        form = form.part(
            "file",
            Part::stream_with_length(stream, content_length)
                .file_name(filename)
                .mime_str(&content_type)?,
        );

        client
            .client
            .post(response.url)
            .multipart(form)
            .send()
            .await?
            .error_for_status()?;

        self.kind = Inner::Uploaded(
            client
                .post(&format!(
                    "project/{}/posts/{}/attach/finish/{}",
                    project, id, response.attachment_id
                ))
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?,
        );
        Ok(())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
struct AttachStartRequest<'a> {
    filename: &'a str,
    content_type: &'a str,
    content_length: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<u32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AttachStartResponse {
    attachment_id: AttachmentId,
    url: String,
    required_fields: HashMap<String, String>,
}
