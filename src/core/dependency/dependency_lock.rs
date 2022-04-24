use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DependencyLock {
    pub url: String,
    pub refname: String,
}
