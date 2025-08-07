use const_format::concatc;

pub mod response;

const ORG_NAME: &'static str = "aftershoot";
const PROJECT: u64 = 24;
const BASE_URL: &'static str = "https://fish.aftershoot.co/api/0/";
const BASE_ORG_URL: &'static str = concatc!(BASE_URL, "organizations/", ORG_NAME, "/");

const ISSUES_URL: &'static str = concatc!(BASE_ORG_URL, "issues/");
const TOKEN: &'static str = std::env!("SENTRY_TOKEN");

pub fn get_user_issues(
    user_email: impl AsRef<str>,
) -> Result<Vec<response::Issue>, reqwest::Error> {
    reqwest::blocking::Client::builder()
        .build()?
        .get(ISSUES_URL)
        .bearer_auth(TOKEN)
        .query(&[("project", PROJECT)])
        .query(&[("statsPeriod", "7d")])
        .query(&[("query", format!("user_email:{}", user_email.as_ref()))])
        .send()?
        .json()
}

pub fn get_issue_events(
    issue_id: u64,
    user_email: impl AsRef<str>,
) -> Result<Vec<response::Event>, reqwest::Error> {
    reqwest::blocking::Client::builder()
        .build()?
        .get(format!("{}{}{}", ISSUES_URL, issue_id, "/events/"))
        .bearer_auth(TOKEN)
        .query(&[("statsPeriod", "7d")])
        .query(&[("query", format!("user_email:{}", user_email.as_ref()))])
        .send()?
        .json()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_issues() {
        let email = "infoccusstudio@gmail.com";

        let res = get_user_issues(email).unwrap();
        dbg!(res);
    }

    #[test]
    fn test_issue_event() {
        let email = "infoccusstudio@gmail.com";

        let res = get_user_issues(email)
            .unwrap()
            .iter()
            .map(|p| get_issue_events(p.id, email))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        dbg!(res);
    }
}
