use crate::{Attachment, Error, Session};
pub(crate) use de::PostPage;
use derive_more::{Display, From, FromStr, Into};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// A post ID.
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
pub struct PostId(pub u64);

/// A project ID.
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
pub struct ProjectId(pub u64);

/// Describes a post's contents.
///
/// When you send a post with [`Session::create_post`] or [`Session::edit_post`], the `Post` must
/// be mutable. This is because the [`attachments`][`Post::attachments`] field will be modified
/// with the ID and URL of the uploaded attachment.
#[derive(Debug, Default)]
#[must_use]
pub struct Post {
    /// Marks the post as [18+ content](https://help.antisoftware.club/support/solutions/articles/62000225024-what-does-adult-content-mean-).
    pub adult_content: bool,
    /// Post headline, which is displayed above attachments and markdown.
    pub headline: String,
    /// List of attachments, displayed between the headline and markdown.
    pub attachments: Vec<Attachment>,
    /// Markdown content for the post, displayed after the headline and attachments.
    pub markdown: String,
    /// List of tags.
    pub tags: Vec<String>,
    /// List of content warnings.
    pub content_warnings: Vec<String>,
    /// Marks the post as a draft, preventing it from being seen by other users without the draft
    /// link.
    pub draft: bool,
    /// Metadata returned by Cohost from posts retrieved from the API.
    pub metadata: Option<PostMetadata>,
}

/// Metadata returned by the Cohost API for posts retrieved from post pages.
#[derive(Debug)]
#[allow(clippy::struct_excessive_bools, clippy::module_name_repetitions)]
pub struct PostMetadata {
    /// All identifiers regarding where this post can be found on Cohost.
    pub locations: PostLocations,
    /// True if the client has permission to share this post.
    pub can_share: bool,
    /// True if adding new comments is disabled on this post.
    pub comments_locked: bool,
    /// True if any contributor to the post is muted by the current account.
    pub has_any_contributor_muted: bool,
    /// True if cohost plus features were available to the poster.
    pub has_cohost_plus: bool,
    /// True if the current account has liked this post.
    pub liked: bool,
    /// The number of comments on this post.
    pub num_comments: u64,
    /// The number of comments on other posts in this post's branch of the
    /// share tree.
    pub num_shared_comments: u64,
    /// True if this post is pinned to its author's profile.
    pub pinned: bool,
    /// The handle of the project that posted this post.
    pub posting_project_id: String,
    /// The time at which the post was published.
    pub publication_date: chrono::DateTime<chrono::Utc>,
    /// A list of the handles of all the projects involved in this post.
    pub related_projects: Vec<String>,
    /// A list of all the posts in this post's branch of the share tree.
    pub share_tree: Vec<Post>,
}

/// All identifying information about where to find a post, from its ID to how to edit it.
#[derive(Debug, Hash, Clone, PartialEq, Eq)]
#[allow(clippy::module_name_repetitions)]
pub struct PostLocations {
    /// The unique numerical ID of the post.
    pub id: PostId,
    /// The filename of the post, excluding the protocol, domain, and project.
    /// Acts as a unique ID with a semi-readable slug.
    pub filename: String,
    /// The complete URL at which this post can be viewed on Cohost.
    pub url: String,
    /// The location at which this post can be edited.
    pub edit_url: String,
}

impl Post {
    /// Returns true if the post has no content (no headline, attachments, or markdown content).
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
        shared_post: Option<PostId>,
    ) -> Result<PostId, Error> {
        if self.is_empty() && shared_post.is_none() {
            return Err(Error::EmptyPost);
        }
        if self.attachments.iter().any(Attachment::is_failed) {
            return Err(Error::FailedAttachment);
        }

        let need_upload = self.attachments.iter().any(Attachment::is_new);

        let de::PostResponse { post_id } = session
            .client
            .request(method, path)
            .json(&self.as_api(need_upload, shared_post))
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
                .json(&self.as_api(false, shared_post))
                .send()
                .await?
                .error_for_status()?;
        }

        Ok(post_id)
    }

    #[tracing::instrument]
    fn as_api(&self, force_draft: bool, shared_post: Option<PostId>) -> ser::Post<'_> {
        let mut blocks = self
            .attachments
            .iter()
            .map(|attachment| ser::Block::Attachment {
                attachment: ser::Attachment {
                    alt_text: &attachment.alt_text,
                    attachment_id: attachment.id().unwrap_or_default(),
                },
            })
            .collect::<Vec<_>>();
        if !self.markdown.is_empty() {
            for block in self.markdown.split("\n\n") {
                blocks.push(ser::Block::Markdown {
                    markdown: ser::Markdown { content: block },
                });
            }
        }

        let post = ser::Post {
            adult_content: self.adult_content,
            blocks,
            cws: &self.content_warnings,
            headline: &self.headline,
            post_state: if force_draft || self.draft { 0 } else { 1 },
            share_of_post_id: shared_post,
            tags: &self.tags,
        };
        tracing::debug!(?post);
        post
    }
}

impl From<de::Post> for Post {
    fn from(api: de::Post) -> Self {
        let locations = PostLocations {
            id: api.post_id,
            filename: api.filename,
            url: api.single_post_page_url,
            edit_url: api.post_edit_url,
        };
        let metadata = PostMetadata {
            locations,
            can_share: api.can_share,
            comments_locked: api.comments_locked,
            has_any_contributor_muted: api.has_any_contributor_muted,
            has_cohost_plus: api.has_cohost_plus,
            liked: api.is_liked,
            num_comments: api.num_comments,
            num_shared_comments: api.num_shared_comments,
            pinned: api.pinned,
            related_projects: {
                let mut related_projects: Vec<String> = api
                    .related_projects
                    .into_iter()
                    .map(|project| project.handle)
                    .collect();
                if related_projects.is_empty() {
                    related_projects.push(api.posting_project.handle.clone());
                };
                related_projects
            },
            posting_project_id: api.posting_project.handle,
            publication_date: api.published_at,
            share_tree: api.share_tree.into_iter().map(Post::from).collect(),
        };

        let attachments: Vec<Attachment> = api
            .blocks
            .into_iter()
            .filter_map(|block| match block {
                de::Block::Attachment { attachment } => {
                    Some(crate::attachment::Attachment::from(attachment))
                }
                de::Block::Markdown { .. } => None,
            })
            .collect();

        Self {
            metadata: Some(metadata),
            adult_content: api.effective_adult_content,
            headline: api.headline,
            markdown: api.plain_text_body,
            tags: api.tags,
            content_warnings: api.cws,
            draft: api.state == 0,
            attachments,
        }
    }
}

impl From<de::Attachment> for Attachment {
    fn from(api: de::Attachment) -> Self {
        Self {
            kind: crate::attachment::Inner::Uploaded(crate::attachment::Finished {
                attachment_id: api.attachment_id,
                url: api.file_url,
            }),
            alt_text: api.alt_text,
        }
    }
}

impl From<PostPage> for Vec<Post> {
    fn from(page: PostPage) -> Self {
        page.items.into_iter().map(Post::from).collect()
    }
}

mod ser {
    use super::PostId;
    use crate::attachment::AttachmentId;
    use serde::Serialize;
    use std::fmt::{self, Debug};

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Post<'a> {
        pub adult_content: bool,
        pub blocks: Vec<Block<'a>>,
        pub cws: &'a [String],
        pub headline: &'a str,
        pub post_state: u64,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub share_of_post_id: Option<PostId>,
        pub tags: &'a [String],
    }

    impl Debug for Post<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", serde_json::to_value(self).map_err(|_| fmt::Error)?)
        }
    }

    #[derive(Serialize)]
    #[serde(tag = "type", rename_all = "camelCase")]
    pub enum Block<'a> {
        Attachment { attachment: Attachment<'a> },
        Markdown { markdown: Markdown<'a> },
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Attachment<'a> {
        pub alt_text: &'a str,
        pub attachment_id: AttachmentId,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Markdown<'a> {
        pub content: &'a str,
    }
}

mod de {
    use super::{PostId, ProjectId};
    use crate::AttachmentId;
    use serde::Deserialize;

    #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PostPage {
        pub(super) n_items: u64,
        pub(super) n_pages: u64,
        pub(super) items: Vec<Post>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
    #[serde(rename_all = "camelCase")]
    #[allow(clippy::struct_excessive_bools)]
    pub struct Post {
        pub blocks: Vec<Block>,
        pub can_publish: bool,
        pub can_share: bool,
        pub comments_locked: bool,
        pub contributor_block_incoming_or_outgoing: bool,
        pub cws: Vec<String>,
        pub effective_adult_content: bool,
        pub filename: String,
        pub has_any_contributor_muted: bool,
        pub has_cohost_plus: bool,
        pub headline: String,
        pub is_editor: bool,
        pub is_liked: bool,
        pub num_comments: u64,
        pub num_shared_comments: u64,
        pub pinned: bool,
        pub plain_text_body: String,
        pub post_edit_url: String,
        pub post_id: PostId,
        pub posting_project: PostingProject,
        pub published_at: chrono::DateTime<chrono::Utc>,
        pub related_projects: Vec<PostingProject>,
        pub share_tree: Vec<Post>,
        pub single_post_page_url: String,
        pub state: u64,
        pub tags: Vec<String>,
        pub transparent_share_of_post_id: Option<PostId>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PostingProject {
        pub handle: String,
        pub display_name: Option<String>,
        pub dek: Option<String>,
        pub description: Option<String>,
        #[serde(rename = "avatarURL")]
        pub avatar_url: String,
        #[serde(rename = "avatarPreviewURL")]
        pub avatar_preview_url: String,
        pub project_id: ProjectId,
        pub privacy: String,
        pub pronouns: Option<String>,
        pub url: Option<String>,
        pub avatar_shape: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
    #[serde(tag = "type", rename_all = "camelCase")]
    pub enum Block {
        Attachment { attachment: Attachment },
        Markdown { markdown: Markdown },
    }

    #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Attachment {
        pub alt_text: String,
        pub attachment_id: AttachmentId,
        #[serde(rename = "fileURL")]
        pub file_url: String,
        #[serde(rename = "previewURL")]
        pub preview_url: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Markdown {
        pub content: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PostResponse {
        pub post_id: PostId,
    }
}

#[test]
fn test_parse_project_post_page() -> Result<(), Box<dyn std::error::Error>> {
    let post_page: de::PostPage =
        serde_json::from_str(include_str!("../samples/example.project.posts.json"))?;
    assert_eq!(post_page.n_items, 3);
    assert_eq!(
        usize::try_from(post_page.n_items).unwrap(),
        post_page.items.len()
    );
    let post = post_page
        .items
        .iter()
        .find(|post| post.post_id.0 == 185_838)
        .expect("Couldn't find post by ID 185838 as expected; did you change the sample?");
    assert_eq!(post.headline, "This is a test post.");
    assert_eq!(post.filename, "185838-this-is-a-test-post");
    assert_eq!(post.state, 1);
    assert!(post.transparent_share_of_post_id.is_none());
    assert_eq!(post.num_comments, 0);
    assert_eq!(post.num_shared_comments, 0);
    assert_eq!(post.tags.len(), 3);
    assert_eq!(post.cws.len(), 0);
    assert_eq!(post.related_projects.len(), 0);

    let post = post_page
        .items
        .iter()
        .find(|post| post.post_id.0 == 185_916)
        .expect("Couldn't find post by ID 185916 as expected; did you change the sample?");

    assert_eq!(post.related_projects.len(), 2);
    assert_eq!(post.share_tree.len(), 1);
    Ok(())
}

#[test]
fn test_convert_post() -> Result<(), Box<dyn std::error::Error>> {
    let post_page: de::PostPage =
        serde_json::from_str(include_str!("../samples/example.project.posts.json"))?;
    let post = post_page
        .items
        .iter()
        .find(|post| post.post_id.0 == 185_838)
        .expect("Couldn't find post by ID 185838 as expected; did you change the sample?");
    let converted_post = Post::from(post.clone());
    let converted_post_metadata = converted_post
        .metadata
        .expect("No metadata for converted post!");
    assert_eq!(post.post_id, converted_post_metadata.locations.id);
    assert!(!converted_post.attachments.is_empty());
    assert_eq!(
        converted_post_metadata.publication_date.timestamp(),
        1_667_531_869
    );

    Ok(())
}
