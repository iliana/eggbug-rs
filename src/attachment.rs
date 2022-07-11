use crate::{Client, Error, PostId};
use bytes::Bytes;
use derive_more::{Display, FromStr};
use reqwest::multipart::{Form, Part};
use reqwest::Body;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[allow(clippy::module_name_repetitions)]
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Deserialize,
    Display,
    Eq,
    FromStr,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
#[serde(transparent)]
pub struct AttachmentId(pub Uuid);

#[derive(Debug)]
pub struct Attachment(Inner);

#[derive(Debug)]
enum Inner {
    New {
        stream: Body,
        filename: String,
        content_type: String,
        content_length: u64,
    },
    Uploaded(Finished),
    Failed,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Finished {
    attachment_id: AttachmentId,
    url: String,
}

impl Attachment {
    /// Create an `Attachment` from a buffer.
    ///
    /// # Panics
    ///
    /// Panics if the length of `content` overflows a [`u64`].
    pub fn new(content: impl Into<Bytes>, filename: String, content_type: String) -> Attachment {
        let content: Bytes = content.into();
        Attachment(Inner::New {
            content_length: content.len().try_into().unwrap(),
            stream: content.into(),
            filename,
            content_type,
        })
    }

    /// Create an `Attachment` from a file on disk.
    #[cfg(feature = "fs")]
    pub async fn new_from_file(
        path: impl AsRef<std::path::Path>,
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

        let file = File::open(path).await?;
        let content_length = file.metadata().await?.len();
        let stream = Body::wrap_stream(FramedRead::new(file, BytesCodec::new()));

        Ok(Attachment(Inner::New {
            stream,
            filename,
            content_type,
            content_length,
        }))
    }

    pub fn is_new(&self) -> bool {
        matches!(self.0, Inner::New { .. })
    }

    pub fn is_uploaded(&self) -> bool {
        matches!(self.0, Inner::Uploaded { .. })
    }

    pub fn is_failed(&self) -> bool {
        matches!(self.0, Inner::Failed)
    }

    pub fn url(&self) -> Option<&str> {
        match &self.0 {
            Inner::Uploaded(Finished { url, .. }) => Some(url),
            _ => None,
        }
    }

    pub(crate) fn id(&self) -> Option<AttachmentId> {
        match self.0 {
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
        let (stream, filename, content_type, content_length) =
            match std::mem::replace(&mut self.0, Inner::Failed) {
                Inner::New {
                    stream,
                    filename,
                    content_type,
                    content_length,
                } => (stream, filename, content_type, content_length),
                Inner::Uploaded(_) => return Ok(()),
                Inner::Failed => return Err(Error::FailedAttachment),
            };

        let response: AttachStartResponse = client
            .post(&format!("project/{}/posts/{}/attach/start", project, id))
            .json(&AttachStartRequest {
                filename: &filename,
                content_type: &content_type,
                content_length: content_length.to_string(),
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

        self.0 = Inner::Uploaded(
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
    content_length: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AttachStartResponse {
    attachment_id: AttachmentId,
    url: String,
    required_fields: HashMap<String, String>,
}
