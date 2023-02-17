use dotenvy::var;
/// Copyright (c) 2023, Sean McNamara <smcnam@gmail.com>.
/// All code in this repository is disjunctively licensed under [CC-BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/) and [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0).
/// Direct dependencies are believed to be under a license which allows downstream code to have these licenses.
use entity::{forum_posts::ForumPost, forum_threads::ForumThread, subforums::Subforum};
use hyper::HeaderMap;
use jsonrpsee::{
    core::{__reexports::serde::Deserialize, client::IdKind},
    http_client::{transport::HttpBackend, HttpClient, HttpClientBuilder},
};
use once_cell::sync::OnceCell;
use reqwest::Client;
use sea_orm::DatabaseConnection;
use secrecy::SecretString;
use std::{collections::HashMap, time::Duration};
use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

lazy_static::lazy_static! {
   pub static ref SEE: HttpClient<tower_http::trace::Trace<HttpBackend, SharedClassifier<ServerErrorsAsFailures>, DefaultMakeSpan>> = {
        let proxy = var("proxy").ok();
        let website = var("website").expect("Required .env variable missing: website");
        let mut headers = HeaderMap::new();
        headers.insert("Accept", "*/*".parse().unwrap());
        headers.insert("User-Agent", "encuum-api".parse().unwrap());
        let middleware = tower::ServiceBuilder::new()
        .layer(
            TraceLayer::new_for_http()
                .on_request(
                    DefaultOnRequest::new().level(Level::DEBUG),
                )
                .on_response(DefaultOnResponse::new().include_headers(true).latency_unit(LatencyUnit::Millis).level(Level::DEBUG)),
        );
        let mut client_builder = HttpClientBuilder::default().set_middleware(middleware).set_headers(headers)
            .id_format(IdKind::String)
            .request_timeout(Duration::from_secs(600))
            .set_max_logging_length(99999999);

        if let Some(prox) = proxy {
            client_builder = client_builder.set_proxy(prox).unwrap();
        }

        client_builder
        .build(format!("https://{}:443/api/v1/api.php", website))
        .unwrap()
    };
}

pub static STATE: OnceCell<State> = OnceCell::new();

#[macro_export]
macro_rules! state {
    () => {
        STATE.get().expect("State not initialized")
    };
}

#[macro_export]
macro_rules! exposed_session {
    () => {
        &state!().session_id.as_ref().unwrap().expose_secret()
    };
}

#[derive(Debug)]
pub struct State {
    pub email: String,
    pub password: SecretString,
    pub session_id: Option<SecretString>,
    pub forum_ids: Option<Vec<String>>,
    pub subforum_ids: Option<Vec<String>>,
    pub keep_going: bool,
    pub do_images: bool,
    pub do_apps: bool,
    pub sanitize_log: bool,
    pub req_client: Client,
    pub conn: DatabaseConnection,
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
    pub notices: Vec<ForumThread>,
    pub announcement_local: Vec<ForumThread>,
    pub announcement_global: Vec<ForumThread>,
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
#[serde(untagged)]
pub enum SubforumType {
    MapSubforum(HashMap<String, Vec<Subforum>>),
    SeqSubforum(Vec<String>),
}

#[derive(Deserialize)]
pub struct GetCafResult {
    pub settings: ForumSettings,
    pub subforums: SubforumType,
    pub total_threads: serde_json::Value,
    pub total_posts: serde_json::Value,
    pub category_names: HashMap<String, String>,
    pub categories: HashMap<String, HashMap<String, Subforum>>,
}

#[derive(Deserialize)]
pub struct GetApplicationsListResult {
    pub items: Option<Vec<MiniApp>>,
    pub total: Option<String>,
}
#[derive(Deserialize)]
pub struct MiniApp {
    pub application_id: Option<String>,
}
