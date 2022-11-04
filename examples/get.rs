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

    let project = std::env::var("COHOST_PROJECT")?;

    let client = Client::new();
    let posts = client.get_posts_page(&project, 0).await?;
    println!("{:#?}", posts);

    Ok(())
}
