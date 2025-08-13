use thiserror::Error;

#[derive(Debug, Error)]
#[error(transparent)]
pub enum Error {
    Reqwest(#[from] reqwest::Error),
    SerdeJson(#[from] serde_json::Error),
    Io(#[from] std::io::Error),
    #[error("discord api error")]
    Discord(u32),
    #[error("member check failed for ckey: {0}")]
    MemberCheckFailed(String),
}
