use crate::{Client, Error, Post, PostId};
use reqwest::Method;

#[derive(Debug, Clone)]
pub struct Session {
    pub(crate) client: Client,
}

impl Session {
    pub async fn login(email: &str, password: &str) -> Result<Session, Error> {
        Client::new().login(email, password).await
    }

    #[tracing::instrument(skip(self))]
    pub async fn create_post(&self, project: &str, post: &mut Post) -> Result<PostId, Error> {
        post.send(
            self,
            Method::POST,
            &format!("project/{}/posts", project),
            project,
        )
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn edit_post(
        &self,
        project: &str,
        id: PostId,
        post: &mut Post,
    ) -> Result<PostId, Error> {
        post.send(
            self,
            Method::PUT,
            &format!("project/{}/posts/{}", project, id),
            project,
        )
        .await
    }

    #[tracing::instrument(skip(self))]
    pub async fn delete_post(&self, project: &str, id: PostId) -> Result<(), Error> {
        self.client
            .delete(&format!("project/{}/posts/{}", project, id))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
