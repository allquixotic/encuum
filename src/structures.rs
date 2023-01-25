/// Copyright (c) 2023, Sean McNamara <smcnam@gmail.com>.
/// All code in this repository is disjunctively licensed under [CC-BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/) and [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0).
/// Direct dependencies such as Rust, Diesel-rs, Hyper and jsonrpsee are licensed under the MIT or 3-clause BSD license, which allow downstream code to have any license.
use std::collections::HashMap;

use diesel::{Insertable, Queryable, SqliteConnection};
use jsonrpsee::{core::__reexports::serde::Deserialize, http_client::HttpClient};
use serde::Serialize;

use crate::schema::*;

pub struct State {
    pub email: String,
    pub password: String,
    pub client: HttpClient,
    pub session_id: Option<String>,
    pub forum_ids: Option<Vec<String>>,
    pub cafs: Option<Vec<GetCafResult>>,
    pub conn: SqliteConnection,
    pub subforum_ids: Option<Vec<String>>,
    pub keep_going: bool,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub session_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ForumSettings {
    pub title_welcome: String,
}

#[derive(Serialize, Deserialize, Insertable)]
pub struct ForumThread {
    pub thread_id: String,
    pub thread_subject: String,
    pub thread_views: String,
    pub thread_type: String,
    pub thread_status: String,
    pub forum_id: String,
    pub username: Option<String>,
    pub category_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetForumResult {
    pub sticky: Vec<ForumThread>,
    pub threads: Vec<ForumThread>,
    pub announcement_local: Vec<ForumThread>,
    pub forum: Subforum,
    pub page: String,
    pub pages: u32,
}

#[derive(Serialize, Deserialize)]
pub struct GetThreadResult {
    pub thread: ForumThread,
    pub posts: Vec<ForumPost>,
    pub total_items: String,
    pub pages: u32,
}

#[derive(Serialize, Deserialize, Insertable)]
pub struct ForumPost {
    pub post_id: String,
    pub post_time: String,
    pub post_content: String,
    pub post_user_id: String,
    pub last_edit_time: String,
    pub post_unhidden: String,
    pub post_admin_hidden: String,
    pub post_locked: String,
    pub last_edit_user: String,
    pub post_username: String,
    pub thread_id: Option<String>,
}

#[derive(Serialize, Deserialize, Insertable)]
pub struct Subforum {
    pub title_welcome: Option<String>,
    pub preset_id: String,
    pub category_id: String,
    pub category_name: String,
    pub forum_id: String,
    pub forum_name: String,
    pub forum_description: String,
    pub parent_id: String,
    pub forum_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetCafResult {
    pub settings: ForumSettings,
    pub subforums: HashMap<String, Vec<Subforum>>,
    pub total_threads: u32,
    pub total_posts: u32,
    pub category_names: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Insertable, Queryable)]
pub struct Image {
    pub image_url: String,
    pub image_content: Vec<u8>,
}
