use records::{Email, ErrorReason};
use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Display},
    ops::Not,
};
use tap::TapOptional;

pub mod api;
pub mod records;

const ERROR_STACK_PATS: &'static [char] = &['├', '╴', '╰', '▶', '│', '─', '┬', '━'];

#[derive(Debug)]
pub struct ParseReasonError;

#[derive(Debug, serde::Serialize)]
pub struct ErrorTree {
    pub inner: (ErrorNode, UserNode),
}

struct _ErrorTreeDisp<'a> {
    inner: (&'a ErrorNode, &'a UserNode),
}

impl ErrorTree {
    pub fn new() -> Self {
        Self {
            inner: (ErrorNode::new(), UserNode::new()),
        }
    }

    pub fn insert_many(&mut self, email: impl ToString, errors: Vec<Vec<ErrorRepr>>) {
        for error in errors {
            self.insert(email.to_string(), error);
        }
    }

    pub fn insert(&mut self, email: impl ToString, errors: Vec<ErrorRepr>) {
        let mut prev = &mut self.inner;
        prev.1.0.insert(email.to_string());

        for error in errors.iter() {
            let nodes = prev
                .0
                .0
                .entry(error.reason.clone())
                .or_insert_with(|| (ErrorNode::new(), UserNode::new()));

            prev = nodes;
            prev.1.0.insert(email.to_string());
        }
    }
}

impl Display for ErrorTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let t = _ErrorTreeDisp {
            inner: (&self.inner.0, &self.inner.1),
        };

        t._display(0, f, &ErrorReason::All)?;

        Ok(())
    }
}

impl<'a> _ErrorTreeDisp<'a> {
    pub fn _display(
        &self,
        mut indent: usize,
        f: &mut fmt::Formatter<'_>,
        parent: &ErrorReason,
    ) -> std::fmt::Result {
        const INDENT: &str = "  ";
        write!(
            f,
            "{}{}({} Users)\n",
            INDENT.repeat(indent),
            parent,
            self.inner.1.0.len()
        )?;
        indent += 1;
        for (reason, (enode, unode)) in &self.inner.0.0 {
            write!(f, "{}{}\n", INDENT.repeat(indent), reason)?;
            let t = Self {
                inner: (enode, unode),
            };
            t._display(indent + 1, f, reason)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ErrorNode(HashMap<ErrorReason, (ErrorNode, UserNode)>);
#[derive(Debug, Clone, serde::Serialize)]
pub struct UserNode(HashSet<Email>);

impl ErrorNode {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

impl UserNode {
    pub fn new() -> Self {
        Self(HashSet::new())
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ErrorRepr {
    pub reason: ErrorReason,
    pub location: Option<String>,
}

impl ErrorRepr {
    pub fn from_msg(msg: impl AsRef<str>) -> Result<Vec<Vec<Self>>, ParseReasonError> {
        let mut res = vec![];
        let mut cur = vec![];
        let mut lines = msg.as_ref().lines().peekable();

        while let Some(line) = lines.next() {
            if line.is_empty() {
                if !cur.is_empty() {
                    res.push(std::mem::take(&mut cur));
                }
                continue;
            };

            let line = line
                .trim_start_matches(|c: char| c.is_whitespace() || ERROR_STACK_PATS.contains(&c));
            if let Some(line) = line.is_empty().not().then_some(line) {
                if let Some(reason) = ErrorReason::from_str(line) {
                    let location = lines
                        .peek()
                        .and_then(|s| {
                            let trimmed = s.trim_start_matches(|c: char| {
                                c.is_whitespace() || ERROR_STACK_PATS.contains(&c)
                            });
                            trimmed.starts_with("at ").then(|| trimmed.to_string())
                        })
                        .tap_some(|_| {
                            lines.next();
                        });

                    let repr = ErrorRepr { reason, location };

                    cur.push(repr);
                }
            }
        }

        if !cur.is_empty() {
            res.push(cur);
        }

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_repr() {
        let msg = r#"
[DownloadError]
├╴at C:\actions-runner\_work\AfterShoot-Desktop-App\AfterShoot-Desktop-App\src\download.rs:1230:10
├╴Reason: [Downloading(NetworkError)]
├╴{"profile_id":100009,"profile_key":"","forcedownload":false,"silent_progress":false,"model_type":"Retouching"}
├╴Error while resolving background task
├╴1 additional opaque attachment
│
├─▶ error decoding response body
│   ╰╴at C:\actions-runner\_work\AfterShoot-Desktop-App\AfterShoot-Desktop-App\src\download.rs:1230:10
│
├─▶ error reading a body from connection
│   ╰╴at C:\actions-runner\_work\AfterShoot-Desktop-App\AfterShoot-Desktop-App\src\download.rs:1230:10
│
╰─▶ Foi forçado o cancelamento de uma conexão existente pelo host remoto. (os error 10054)
    ├╴at C:\actions-runner\_work\AfterShoot-Desktop-App\AfterShoot-Desktop-App\src\download.rs:1230:10
    ╰╴span trace with 2 frames (1)
"#;
        let repr = ErrorRepr::from_msg(msg).unwrap();
        dbg!(repr);
    }

    #[test]
    fn test_error_tree() {
        let email = "XDD";
        let msg = r#"
[DownloadError]
├╴at C:\actions-runner\_work\AfterShoot-Desktop-App\AfterShoot-Desktop-App\src\download.rs:1230:10
├╴Reason: [Downloading(NetworkError)]
├╴{"profile_id":100009,"profile_key":"","forcedownload":false,"silent_progress":false,"model_type":"Retouching"}
├╴Error while resolving background task
├╴1 additional opaque attachment
│
├─▶ error decoding response body
│   ╰╴at C:\actions-runner\_work\AfterShoot-Desktop-App\AfterShoot-Desktop-App\src\download.rs:1230:10
│
├─▶ error reading a body from connection
│   ╰╴at C:\actions-runner\_work\AfterShoot-Desktop-App\AfterShoot-Desktop-App\src\download.rs:1230:10
│
╰─▶ Foi forçado o cancelamento de uma conexão existente pelo host remoto. (os error 10054)
    ├╴at C:\actions-runner\_work\AfterShoot-Desktop-App\AfterShoot-Desktop-App\src\download.rs:1230:10
    ╰╴span trace with 2 frames (1)
"#;
        let errors = ErrorRepr::from_msg(msg).unwrap();

        let mut tree = ErrorTree::new();

        tree.insert_many(email, errors);

        println!("{}", tree);
    }
}
