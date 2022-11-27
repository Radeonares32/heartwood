use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::prelude::*;

/// Author.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Author {
    pub id: NodeId,
}

impl Author {
    pub fn new(id: NodeId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> &NodeId {
        &self.id
    }
}

#[derive(Debug, Default, Copy, Clone, PartialOrd, PartialEq, Ord, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp {
    seconds: u64,
}

impl Timestamp {
    pub fn new(seconds: u64) -> Self {
        Self { seconds }
    }

    pub fn now() -> Self {
        #[allow(clippy::unwrap_used)] // Safe because Unix was already invented!
        let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        Self {
            seconds: duration.as_secs(),
        }
    }

    pub fn as_secs(&self) -> u64 {
        self.seconds
    }
}

impl std::ops::Add<u64> for Timestamp {
    type Output = Self;

    fn add(self, rhs: u64) -> Self::Output {
        Self {
            seconds: self.seconds + rhs,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ReactionError {
    #[error("invalid reaction")]
    InvalidReaction,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Reaction {
    pub emoji: char,
}

impl Reaction {
    pub fn new(emoji: char) -> Result<Self, ReactionError> {
        if emoji.is_whitespace() || emoji.is_ascii() || emoji.is_alphanumeric() {
            return Err(ReactionError::InvalidReaction);
        }
        Ok(Self { emoji })
    }
}

impl FromStr for Reaction {
    type Err = ReactionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.chars();
        let first = chars.next().ok_or(ReactionError::InvalidReaction)?;

        // Reactions should not consist of more than a single emoji.
        if chars.next().is_some() {
            return Err(ReactionError::InvalidReaction);
        }
        Reaction::new(first)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum TagError {
    #[error("invalid tag name: `{0}`")]
    InvalidName(String),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Tag(String);

impl Tag {
    pub fn new(name: impl Into<String>) -> Result<Self, TagError> {
        let name = name.into();

        if name.chars().any(|c| c.is_whitespace()) || name.is_empty() {
            return Err(TagError::InvalidName(name));
        }
        Ok(Self(name))
    }

    pub fn name(&self) -> &str {
        self.0.as_str()
    }
}

impl FromStr for Tag {
    type Err = TagError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl From<Tag> for String {
    fn from(Tag(name): Tag) -> Self {
        name
    }
}

/// RGB color.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Color(u32);

#[derive(thiserror::Error, Debug)]
pub enum ColorConversionError {
    #[error("invalid format: expect '#rrggbb'")]
    InvalidFormat,
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:06x}", self.0)
    }
}

impl FromStr for Color {
    type Err = ColorConversionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex = s.replace('#', "").to_lowercase();

        if hex.chars().count() != 6 {
            return Err(ColorConversionError::InvalidFormat);
        }

        match u32::from_str_radix(&hex, 16) {
            Ok(n) => Ok(Color(n)),
            Err(e) => Err(e.into()),
        }
    }
}

/// A discussion thread.
pub type Discussion = Vec<Comment<Replies>>;

/// Comment replies.
pub type Replies = Vec<Comment>;

/// Local id of a comment in an issue.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct CommentId {
    /// Represents the index of the comment in the thread,
    /// with `0` being the top-level comment.
    ix: usize,
}

impl CommentId {
    /// Root comment.
    pub const fn root() -> Self {
        Self { ix: 0 }
    }
}

impl From<usize> for CommentId {
    fn from(ix: usize) -> Self {
        Self { ix }
    }
}

impl From<CommentId> for usize {
    fn from(id: CommentId) -> Self {
        id.ix
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment<R = ()> {
    pub author: Author,
    pub body: String,
    pub reactions: HashMap<Reaction, usize>,
    pub replies: R,
    pub timestamp: Timestamp,
}

impl<R: Default> Comment<R> {
    pub fn new(author: Author, body: String, timestamp: Timestamp) -> Self {
        Self {
            author,
            body,
            reactions: HashMap::default(),
            replies: R::default(),
            timestamp,
        }
    }
}