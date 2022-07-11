use anyhow::{Context, Result};
use eggbug::{PostId, Session};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let email = std::env::var("COHOST_EMAIL")?;
    let password = std::env::var("COHOST_PASSWORD")?;
    let project = std::env::var("COHOST_PROJECT")?;
    let post_id = PostId(
        std::env::args()
            .nth(1)
            .context("usage: delete POST_ID")?
            .parse()
            .context("failed to parse post ID")?,
    );

    let session = Session::login(&email, &password).await?;
    session.delete_post(&project, post_id).await?;

    Ok(())
}
