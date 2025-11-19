use std::{collections::HashMap, num::ParseIntError};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "IssueInterm")]
pub struct Issue {
    pub id: u64,
}

#[derive(serde::Deserialize)]
struct IssueInterm {
    pub id: String,
}

impl TryFrom<IssueInterm> for Issue {
    type Error = ParseIntError;

    fn try_from(value: IssueInterm) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.parse()?,
        })
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(from = "EventInterm")]
pub struct Event {
    pub id: String,
    pub message: String,
    pub title: String,
    pub tags: HashMap<String, String>,
    #[serde(flatten)]
    pub rest: serde_json::Value,
}

#[derive(Debug, serde::Deserialize)]
struct EventInterm {
    pub id: String,
    pub message: String,
    pub title: String,
    pub tags: Vec<Tag>,
    #[serde(flatten)]
    pub rest: serde_json::Value,
}

#[derive(Debug, serde::Deserialize)]
struct Tag {
    key: String,
    value: String,
}

impl From<EventInterm> for Event {
    fn from(value: EventInterm) -> Self {
        Self {
            id: value.id,
            message: value.message,
            title: value.title,
            tags: value.tags.into_iter().map(|x| (x.key, x.value)).collect(),
            rest: value.rest,
        }
    }
}
