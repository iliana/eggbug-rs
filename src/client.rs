use crate::{Error, Post, Session};
use reqwest::{Method, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

const PBKDF2_ITERATIONS: u32 = 200_000;
const PBKDF2_KEY_LENGTH: usize = 128;

macro_rules! request_impl {
    ($($f:ident),* $(,)*) => {
        $(
            #[inline]
            pub(crate) fn $f(&self, path: &str) -> RequestBuilder {
                tracing::info!(path, concat!("Client::", stringify!($f)));
                self.client.$f(format!("{}{}", self.base_url, path))
            }
        )*
    };
}

/// HTTP client.
#[derive(Debug, Clone)]
pub struct Client {
    pub(crate) base_url: Cow<'static, str>,
    pub(crate) client: reqwest::Client,
}

impl Client {
    /// Creates a new `Client` with the default base URL, `https://cohost.org/api/v1/`. Use
    /// [`Client::with_base_url`] to change the base URL.
    #[must_use]
    #[allow(clippy::missing_panics_doc)] // tested to not panic
    pub fn new() -> Client {
        const USER_AGENT: &str = concat!(
            "eggbug-rs/",
            env!("CARGO_PKG_VERSION"),
            " (https://github.com/iliana/eggbug-rs)",
        );

        Client {
            base_url: Cow::Borrowed("https://cohost.org/api/v1/"),
            client: reqwest::Client::builder()
                .cookie_store(true)
                .user_agent(USER_AGENT)
                .build()
                .unwrap(),
        }
    }

    /// Creates a new `Client` with a custom base URL.
    #[must_use]
    pub fn with_base_url(mut self, mut base_url: String) -> Client {
        if !base_url.ends_with('/') {
            base_url.push('/');
        }
        self.base_url = Cow::Owned(base_url);
        self
    }

    /// Logs into cohost with an email and password, returning a [`Session`].
    ///
    /// Securely storing the user's password is an exercise left to the caller.
    #[tracing::instrument(skip(self, password))]
    pub async fn login(self, email: &str, password: &str) -> Result<Session, Error> {
        let SaltResponse { salt } = self
            .get("login/salt")
            .query(&[("email", email)])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let mut client_hash = [0; PBKDF2_KEY_LENGTH];
        pbkdf2::pbkdf2::<hmac::Hmac<sha2::Sha384>>(
            password.as_bytes(),
            &decode_salt(&salt)?,
            PBKDF2_ITERATIONS,
            &mut client_hash,
        );
        let client_hash = base64::encode(&client_hash);

        let LoginResponse { user_id } = self
            .post("login")
            .json(&LoginRequest { email, client_hash })
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        tracing::info!(user_id, "logged in");

        Ok(Session { client: self })
    }

    /// Get a page of posts from the given project.
    ///
    /// Pages start at 0. Once you get an empty page, there are no more pages after that to get; they will all be empty.
    #[tracing::instrument(skip(self))]
    pub async fn get_posts_page(&self, project: &str, page: u64) -> Result<Vec<Post>, Error> {
        let posts_page: crate::post::PostPage = self
            .get(&format!("project/{}/posts", project))
            .query(&[("page", page.to_string())])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(posts_page.into())
    }

    #[inline]
    pub(crate) fn request(&self, method: Method, path: &str) -> RequestBuilder {
        tracing::info!(%method, path, "Client::request");
        self.client
            .request(method, format!("{}{}", self.base_url, path))
    }

    request_impl!(delete, get, post, put);
}

impl Default for Client {
    fn default() -> Client {
        Client::new()
    }
}

/// There is a subtle bug(?) in cohost:
/// - The salt returned from the `login/salt` endpoint returns a string that _appears_ to be
///   using the URL-safe Base64 alphabet with no padding.
/// - However, the salt is being decoded with some JavaScript code that uses the standard
///   (`+/`) alphabet.
/// - This code uses a lookup table to go from a Base64 character to a 6-bit value. If the
///   character is not in the lookup table, the lookup returns `undefined`. The code then
///   performs bitwise operations on the returned value, which is coerced to 0 if not present
///   in the lookup table.
///
/// We can replicate this effect by replacing hyphens and underscores with the `A`, the
/// Base64 character representing 0.
///
/// mogery seemed to know about this when writing cohost.js (see lib/b64arraybuffer.js):
/// <https://github.com/mogery/cohost.js/commit/c0063a38ae334b4424989242821d0ac1aba78f03>
fn decode_salt(salt: &str) -> Result<Vec<u8>, Error> {
    Ok(base64::decode_config(
        &salt.replace(['-', '_'], "A"),
        base64::STANDARD_NO_PAD,
    )?)
}

#[cfg(test)]
#[test]
fn test_decode_salt() {
    assert_eq!(
        decode_salt("JGhosofJGYFsyBlZspFVYg").unwrap(),
        base64::decode_config("JGhosofJGYFsyBlZspFVYg", base64::URL_SAFE_NO_PAD).unwrap()
    );
    assert_eq!(
        decode_salt("dg6y2aIj_iKzcgaL_MM8_Q").unwrap(),
        base64::decode_config("dg6y2aIjAiKzcgaLAMM8AQ", base64::URL_SAFE_NO_PAD).unwrap()
    );
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaltResponse {
    salt: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginRequest<'a> {
    email: &'a str,
    client_hash: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoginResponse {
    user_id: u64,
}

#[cfg(test)]
mod tests {
    use super::Client;

    #[test]
    fn client_new_doesnt_panic() {
        drop(Client::new());
    }
}
