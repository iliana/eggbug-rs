#![deny(elided_lifetimes_in_paths)]
#![warn(clippy::pedantic)]

use anyhow::Result;
use eggbug::{Attachment, Client, Post};
use std::path::Path;
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
        headline: "test from eggbug-rs".into(),
        attachments: vec![
            Attachment::new_from_file(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("examples")
                    .join("uh-oh.png"),
                "image/png".into(),
            )
            .await?,
        ],
        draft: false,
        ..Default::default()
    };
    let id = session.create_post(&project, &mut post).await?;

    post.markdown = "yahoo\n\n---\n\nread more works!".into();
    session.edit_post(&project, id, &mut post).await?;

    Ok(())
}
