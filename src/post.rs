use crate::{Attachment, AttachmentId, Error, Session};
use derive_more::{Display, FromStr};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug};

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
pub struct PostId(pub u64);

#[derive(Debug, Default)]
#[must_use]
pub struct Post {
    pub adult_content: bool,
    pub attachments: Vec<Attachment>,
    pub content_warnings: Vec<String>,
    pub draft: bool,
    pub headline: String,
    pub markdown: String,
    pub tags: Vec<String>,
}

impl Post {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.attachments.is_empty() && self.headline.is_empty() && self.markdown.is_empty()
    }

    pub(crate) async fn send(
        &mut self,
        session: &Session,
        method: Method,
        path: &str,
        project: &str,
    ) -> Result<PostId, Error> {
        if self.is_empty() {
            return Err(Error::EmptyPost);
        }
        if self.attachments.iter().any(Attachment::is_failed) {
            return Err(Error::FailedAttachment);
        }

        let need_upload = self.attachments.iter().any(Attachment::is_new);

        let PostResponse { post_id } = session
            .client
            .request(method, path)
            .json(&self.as_api(need_upload))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        tracing::info!(%post_id);

        if need_upload {
            futures::future::try_join_all(
                self.attachments
                    .iter_mut()
                    .map(|attachment| attachment.upload(&session.client, project, post_id)),
            )
            .await?;

            session
                .client
                .put(&format!("project/{}/posts/{}", project, post_id))
                .json(&self.as_api(false))
                .send()
                .await?
                .error_for_status()?;
        }

        Ok(post_id)
    }

    #[tracing::instrument]
    fn as_api(&self, force_draft: bool) -> ApiPost<'_> {
        let mut blocks = self
            .attachments
            .iter()
            .map(|attachment| ApiBlock::Attachment {
                attachment: ApiAttachment {
                    attachment_id: attachment.id(),
                },
            })
            .collect::<Vec<_>>();
        if !self.markdown.is_empty() {
            for block in self.markdown.split("\n\n") {
                blocks.push(ApiBlock::Markdown {
                    markdown: ApiMarkdown { content: block },
                });
            }
        }

        let post = ApiPost {
            adult_content: self.adult_content,
            blocks,
            cws: &self.content_warnings,
            headline: &self.headline,
            post_state: if force_draft || self.draft { 0 } else { 1 },
            tags: &self.tags,
        };
        tracing::debug!(?post);
        post
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiPost<'a> {
    adult_content: bool,
    blocks: Vec<ApiBlock<'a>>,
    cws: &'a [String],
    headline: &'a str,
    post_state: u64,
    tags: &'a [String],
}

impl Debug for ApiPost<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_value(self).map_err(|_| fmt::Error)?)
    }
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum ApiBlock<'a> {
    Attachment { attachment: ApiAttachment },
    Markdown { markdown: ApiMarkdown<'a> },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiAttachment {
    #[serde(serialize_with = "serialize_attachment_id")]
    attachment_id: Option<AttachmentId>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiMarkdown<'a> {
    content: &'a str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PostResponse {
    post_id: PostId,
}

fn serialize_attachment_id<S>(id: &Option<AttachmentId>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match id {
        Some(id) => id.serialize(serializer),
        None => "".serialize(serializer),
    }
}

#[cfg(test)]
mod tests {
    use super::ApiAttachment;
    use crate::AttachmentId;
    use serde_json::json;
    use uuid::uuid;

    #[test]
    fn test_serialize_attachment() {
        assert_eq!(
            serde_json::to_value(&ApiAttachment {
                attachment_id: Some(AttachmentId(uuid!("92bfaa11-8e42-4f60-acf4-6fd714b5678b")))
            })
            .unwrap(),
            json!({
                "attachmentId": "92bfaa11-8e42-4f60-acf4-6fd714b5678b",
            })
        );
        assert_eq!(
            serde_json::to_value(&ApiAttachment {
                attachment_id: None
            })
            .unwrap(),
            json!({
                "attachmentId": "",
            })
        );
    }
}
