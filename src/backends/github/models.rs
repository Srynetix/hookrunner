use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct PingEvent<'a> {
    pub zen: &'a str,
    pub repository: Repository<'a>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Repository<'a> {
    pub full_name: &'a str,
    pub name: &'a str,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PushEvent<'a> {
    #[serde(rename = "ref")]
    pub reference: &'a str,
    pub base_ref: &'a str,
    pub head_commit: Commit<'a>,
    pub repository: Repository<'a>,
    pub pusher: CommitUser<'a>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Commit<'a> {
    pub message: &'a str,
    pub timestamp: &'a str,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CommitUser<'a> {
    pub name: &'a str,
    pub email: &'a str,
}
