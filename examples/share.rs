#![deny(elided_lifetimes_in_paths)]
#![warn(clippy::pedantic)]

use anyhow::Result;
use eggbug::{Client, Post};
use tracing_subscriber::{fmt, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let email = std::env::var("COHOST_EMAIL")?;
    let password = std::env::var("COHOST_PASSWORD")?;
    let project = std::env::var("COHOST_PROJECT")?;

    let client = Client::new();
    let session = client.login(&email, &password).await?;

    let mut post = Post {
        markdown: "wow".into(),
        ..Default::default()
    };
    session
        .share_post(&project, 59547.into(), &mut post)
        .await?;

    Ok(())
}
