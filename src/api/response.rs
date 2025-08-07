use std::num::ParseIntError;

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
pub struct Event {
    pub id: String,
    pub message: String,
    pub title: String,
}
