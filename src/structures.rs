/// Copyright (c) 2023, Sean McNamara <smcnam@gmail.com>.
/// All code in this repository is disjunctively licensed under [CC-BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/) and [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0).
/// Direct dependencies such as Rust, Diesel-rs, Hyper and jsonrpsee are licensed under the MIT or 3-clause BSD license, which allow downstream code to have any license.
use std::collections::HashMap;

use jsonrpsee::{core::__reexports::serde::Deserialize, http_client::HttpClient};
use reqwest::Client;
use sea_orm::DatabaseConnection;

use entity::{forum_posts::ForumPost, forum_threads::ForumThread, subforums::Subforum};

pub struct State {
    pub email: String,
    pub password: String,
    pub client: HttpClient,
    pub session_id: Option<String>,
    pub forum_ids: Option<Vec<String>>,
    pub cafs: Option<Vec<GetCafResult>>,
    pub conn: DatabaseConnection,
    pub subforum_ids: Option<Vec<String>>,
    pub keep_going: bool,
    pub req_client: Client,
}

#[derive(Deserialize)]
pub struct LoginResponse {
    pub session_id: String,
}

#[derive(Deserialize)]
pub struct ForumSettings {
    pub title_welcome: String,
}

#[derive(Deserialize, Debug)]
pub struct GetForumResult {
    pub sticky: Vec<ForumThread>,
    pub threads: Vec<ForumThread>,
    pub announcement_local: Vec<ForumThread>,
    pub forum: Subforum,
    pub page: serde_json::Value,
    pub pages: serde_json::Value,
}

#[derive(Deserialize)]
pub struct GetThreadResult {
    pub thread: ForumThread,
    pub posts: Vec<ForumPost>,
    pub total_items: serde_json::Value,
    pub pages: serde_json::Value,
}

#[derive(Deserialize)]
pub struct GetCafResult {
    pub settings: ForumSettings,
    pub subforums: HashMap<String, Vec<Subforum>>,
    pub total_threads: serde_json::Value,
    pub total_posts: serde_json::Value,
    pub category_names: HashMap<String, String>,
    pub categories: HashMap<String, HashMap<String, Subforum>>,
}
