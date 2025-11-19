use std::fmt::{Debug, Display};

use const_format::concatc;

pub mod response;

const ORG_NAME: &'static str = "aftershoot";
const PROJECT: u64 = 24;
const BASE_URL: &'static str = "https://fish.aftershoot.co/api/0/";
const BASE_ORG_URL: &'static str = concatc!(BASE_URL, "organizations/", ORG_NAME, "/");

const ISSUES_URL: &'static str = concatc!(BASE_ORG_URL, "issues/");

pub const TOKEN_ENV_NAME: &'static str = "SENTRY_TOKEN";

#[tracing::instrument(skip(token))]
pub fn get_user_issues(
    user_email: impl AsRef<str> + Debug,
    token: impl Display,
    period: impl AsRef<str> + Debug,
) -> Result<Vec<response::Issue>, reqwest::Error> {
    tracing::info!("Get issues for user");
    reqwest::blocking::Client::builder()
        .build()?
        .get(ISSUES_URL)
        .bearer_auth(token)
        .query(&[("project", PROJECT)])
        .query(&[("statsPeriod", period.as_ref())])
        .query(&[("query", format!("user_email:{}", user_email.as_ref()))])
        .send()?
        .json()
}

#[tracing::instrument(skip(token))]
pub fn get_issues_with_query(
    query: impl AsRef<str> + Debug,
    token: impl Display,
    period: impl AsRef<str> + Debug,
) -> Result<Vec<response::Issue>, reqwest::Error> {
    tracing::info!("Get issues for query");
    reqwest::blocking::Client::builder()
        .build()?
        .get(ISSUES_URL)
        .bearer_auth(token)
        .query(&[("project", PROJECT)])
        .query(&[("statsPeriod", period.as_ref())])
        .query(&[("query", query.as_ref())])
        .send()?
        .json()
}

#[tracing::instrument(skip(token))]
pub fn get_issue_events_for_user(
    issue_id: u64,
    user_email: impl AsRef<str> + Debug,
    token: impl Display,
    period: impl AsRef<str> + Debug,
) -> Result<Vec<response::Event>, reqwest::Error> {
    tracing::info!("Get issue events");
    reqwest::blocking::Client::builder()
        .build()?
        .get(format!("{}{}{}", ISSUES_URL, issue_id, "/events/"))
        .bearer_auth(token)
        .query(&[("dataset", "errors")])
        .query(&[("statsPeriod", period.as_ref())])
        .query(&[("query", format!("user_email:{}", user_email.as_ref()))])
        .send()?
        .json()
}

#[tracing::instrument(skip(token))]
pub fn get_issue_events(
    issue_id: u64,
    token: impl Display,
    period: impl AsRef<str> + Debug,
) -> Result<Vec<response::Event>, reqwest::Error> {
    tracing::info!("Get events for issue");
    reqwest::blocking::Client::builder()
        .build()?
        .get(format!("{}{}{}", ISSUES_URL, issue_id, "/events/"))
        .bearer_auth(token)
        .query(&[("dataset", "errors")])
        .query(&[("statsPeriod", period.as_ref())])
        .send()?
        .json()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_issues() {
        let email = "infoccusstudio@gmail.com";
        let token = std::env!("SENTRY_TOKEN");
        let period = "7d";

        let res = get_user_issues(email, token, period).unwrap();
        dbg!(res);
    }

    #[test]
    fn test_query_issues() {
        let token = std::env!("SENTRY_TOKEN");
        let period = "90d";
        let query = "ndcv";

        let res = get_issues_with_query(query, token, period).unwrap();
        dbg!(&res);
        dbg!(res.len());
    }

    #[test]
    fn test_user_issue_events() {
        let email = "verenapoeschl13@gmail.com";
        let token = std::env!("SENTRY_TOKEN");
        let period = "7d";

        let res = get_user_issues(email, token, period)
            .unwrap()
            .iter()
            .map(|p| get_issue_events_for_user(p.id, email, token, period))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        dbg!(res);
    }

    #[test]
    fn test_issue_events() {
        let token = std::env!("SENTRY_TOKEN");
        let period = "7d";

        let res = get_issues_with_query("ndcv", token, period)
            .unwrap()
            .iter()
            .map(|p| get_issue_events(p.id, token, period))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        dbg!(res);
    }
}
