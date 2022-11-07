use crate::{Client, Error, Post, PostId};
use reqwest::Method;

/// Logged-in session.
#[derive(Debug, Clone)]
pub struct Session {
    pub(crate) client: Client,
}

impl Session {
    /// Returns the inner [`Client`] for this session.
    ///
    /// This can be used to access methods present on `Client`.
    #[must_use]
    pub fn as_client(&self) -> &Client {
        &self.client
    }

    /// Logs into cohost with an email and password, returning a `Session`.
    ///
    /// Securely storing the user's password is an exercise left to the caller.
    pub async fn login(email: &str, password: &str) -> Result<Session, Error> {
        Client::new().login(email, password).await
    }

    /// Create a post.
    ///
    /// Returns the new post's ID.
    #[tracing::instrument(skip(self))]
    pub async fn create_post(&self, page: &str, post: &mut Post) -> Result<PostId, Error> {
        post.send(
            self,
            Method::POST,
            &format!("project/{}/posts", page),
            page,
            None,
        )
        .await
    }

    /// Share a post.
    ///
    /// Returns the new post's ID.
    ///
    /// To share a post with no additional content, use `Post::default()` for `post`.
    #[tracing::instrument(skip(self))]
    pub async fn share_post(
        &self,
        page: &str,
        shared_post: PostId,
        post: &mut Post,
    ) -> Result<PostId, Error> {
        post.send(
            self,
            Method::POST,
            &format!("project/{}/posts", page),
            page,
            Some(shared_post),
        )
        .await
    }

    /// Edit a post.
    ///
    /// Returns the edited post's ID.
    #[tracing::instrument(skip(self))]
    pub async fn edit_post(
        &self,
        page: &str,
        id: PostId,
        post: &mut Post,
    ) -> Result<PostId, Error> {
        post.send(
            self,
            Method::PUT,
            &format!("project/{}/posts/{}", page, id),
            page,
            None,
        )
        .await
    }

    /// Delete a post.
    #[tracing::instrument(skip(self))]
    pub async fn delete_post(&self, page: &str, id: PostId) -> Result<(), Error> {
        self.client
            .delete(&format!("project/{}/posts/{}", page, id))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
