use std::{fmt::Display, io, ops::Not};

use serde::{Deserialize, Serialize};

pub type Email = String;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
#[serde(untagged)]
pub enum ErrorReason {
    DownloadingNetworkDownloadTimeout,
    DownloadingOtherError,
    DownloadingNetworkConnectionError,
    DownloadingInsufficientPermissions,
    DownloadingNetworkFailedToGetModelDownloadUrls,
    DownloadingNetworkError,
    ErrorDecodingResponseBody,
    ErrorReadingBodyFromConnection,
    ErrorSendingRequestForUrl,
    UnexpectedEOFDuringHandshake,
    GenericCancelException,
    DownloadError,
    OSError(i64),
    OtherError,
    Unknown(String),
    All,
}

impl Display for ErrorReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown(s) => write!(f, "[{s}]"),
            Self::OSError(code) => write!(f, "OS error {code}"),
            s => write!(f, "{s:?}"),
        }
    }
}

impl ErrorReason {
    pub fn from_str(s: impl AsRef<str>) -> Option<ErrorReason> {
        const ERR_PATS: &'static [char] = &['[', ']'];
        let s = s
            .as_ref()
            .trim_matches(|c: char| ERR_PATS.contains(&c) || c.is_whitespace());

        if s.contains("Downloading(NetworkError)") {
            return Some(Self::DownloadingNetworkError);
        } else if s.contains("DownloadError") {
            return Some(Self::DownloadError);
        } else if s.contains("Downloading(NetworkConnectionError)") {
            return Some(Self::DownloadingNetworkConnectionError);
        } else if s.contains("Downloading(NetworkDownloadTimeout)") {
            return Some(Self::DownloadingNetworkDownloadTimeout);
        } else if s.contains("error decoding response body") {
            return Some(Self::ErrorDecodingResponseBody);
        } else if s.contains("error reading a body from connection") {
            return Some(Self::ErrorReadingBodyFromConnection);
        } else if s.contains("os error") {
            return Some(Self::OSError(
                s.split("os error ")
                    .skip(1)
                    .next()
                    .and_then(|s| s.split_whitespace().next())
                    .map(|code| {
                        code.trim_matches(|c: char| c.is_whitespace() || c.is_numeric().not())
                            .parse()
                            .ok()
                    })
                    .flatten()?,
            ));
        } else if s.contains("Other Error") {
            return Some(Self::OtherError);
        } else if s.contains("error sending request for url") {
            return Some(Self::ErrorSendingRequestForUrl);
        } else if s.contains("unexpected EOF during handshake") {
            return Some(Self::UnexpectedEOFDuringHandshake);
        } else if s.contains("Generic(CancelException)") {
            return Some(Self::GenericCancelException);
        } else if s.contains("span trace") {
            return None;
        } else if s.contains("Error while resolving background task") {
            return None;
        } else if s.starts_with('{') {
            return None;
        } else if s.contains("opaque attachment") {
            return None;
        } else if s.starts_with("at ") {
            return None;
        } else if s.contains("Correlation Id") {
            return None;
        } else {
            return Some(Self::Unknown(s.to_string()));
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, Hash)]
pub struct UserRecord {
    email: Email,
}

#[derive(Debug, Default)]
pub struct Records {
    inner: Vec<UserRecord>,
}

impl Records {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn by_users(self) -> Vec<Email> {
        self.inner.into_iter().map(|x| x.email).collect()
    }
}

impl<R: io::Read> TryFrom<&mut csv::Reader<R>> for Records {
    type Error = csv::Error;
    fn try_from(rdr: &mut csv::Reader<R>) -> Result<Self, Self::Error> {
        let mut records = Self::new();

        for res in rdr.deserialize() {
            let record: UserRecord = res?;

            records.inner.push(record);
        }

        Ok(records)
    }
}
