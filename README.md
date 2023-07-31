# eggbug

eggbug-rs is a bot library for [cohost.org](https://cohost.org/rc/welcome), providing an
interface to create, read, edit, and delete posts.

```rust
use eggbug::{Post, Session};

// Log in
let session = Session::login("eggbug@website.invalid", "hunter2").await?;

// Describe a post
let mut post = Post {
    headline: "hello from eggbug-rs!".into(),
    markdown: "wow it's like a website in here".into(),
    ..Default::default()
};

// Create the post on the eggbug page
let id = session.create_post("eggbug", &mut post).await?;

// Oh wait we want to make that a link
post.markdown = "wow it's [like a website in here](https://cohost.org/hthrflwrs/post/25147-empty)".into();
session.edit_post("eggbug", id, &mut post).await?;

// Good job!
```

## License

eggbug-rs is released under the terms of the Anti-Capitalist Software License, version 1.4.

## Maintenance

eggbug-rs is "lightly maintained": pull requests are generally merged quickly and without
testing or API review, and new releases will generally be "breaking" (e.g. 0.2.x -> 0.3.x).
