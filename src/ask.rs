use derive_more::{Display, From, FromStr, Into};
use serde::{Deserialize, Serialize};

/// An ask ID.
#[derive(
    Clone,
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
pub struct AskId(pub String);

/// Describes the contents of an ask. Asks can't be created client-side, only decoded when reading
/// content from the server.
#[derive(Clone, Debug)]
pub struct Ask {
    pub(crate) ask_id: AskId,
    /// Information about the account that sent this ask, if it wasn't sent anonymously.
    pub asker: Option<Asker>,
    /// Markdown content for the ask, displayed after the asker's name.
    pub content: String,
    /// The date and time this ask was sent.
    pub sent_at: chrono::DateTime<chrono::Utc>,
}

impl Ask {
    /// Get the ID of the ask represented by this struct.
    pub fn id(&self) -> &str {
        &self.ask_id.0
    }
}

/// Describes the project that sent an ask.
#[derive(Clone, Debug)]
pub struct Asker {
    /// The unique handle of the asker.
    pub handle: String,
    /// The display name of the asker, which may be different from the handle.
    pub display_name: String,
}
