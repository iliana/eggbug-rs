use crate::{Client, Error, Post, PostId};
use reqwest::Method;

/// Logged-in session.
#[derive(Debug, Clone)]
pub struct Session {
    pub(crate) client: Client,
}

impl Session {
    /// Logs into cohost with an email and password, returning a `Session`.
    ///
    /// Securely storing the user's password is an exercise left to the caller.
    pub async fn login(email: &str, password: &str) -> Result<Session, Error> {
        Client::new().login(email, password).await
    }

    /// Create a post.
    #[tracing::instrument(skip(self))]
    pub async fn create_post(&self, page: &str, post: &mut Post) -> Result<PostId, Error> {
        post.send(self, Method::POST, &format!("project/{}/posts", page), page)
            .await
    }

    /// Edit a post.
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
