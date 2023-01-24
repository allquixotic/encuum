pub mod forum;
pub mod schema;
pub mod structures;

use crate::forum::*;
use crate::structures::*;
use diesel::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenvy::var;
use jsonrpsee::{core::client::IdKind, http_client::HttpClientBuilder, ws_client::HeaderMap};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

impl State {
    pub fn new() -> Self {
        let proxy = var("proxy").ok();
        let website = var("website").expect("Required .env variable missing: website");
        let mut headers = HeaderMap::new();
        headers.insert("Accept", "*/*".parse().unwrap());
        headers.insert("User-Agent", "encuum-api".parse().unwrap());
        let mut client_builder = HttpClientBuilder::default()
            .set_headers(headers)
            .id_format(IdKind::String);
        if proxy.as_ref().is_some() {
            client_builder = client_builder.set_proxy(proxy.as_ref().unwrap()).unwrap();
        }

        let forum_preset_ids = var("forum_ids").ok();
        let forum_ids: Option<Vec<String>> = match forum_preset_ids {
            Some(fpis) => Some(fpis.split(",").map(|s| s.to_string()).collect()),
            None => None,
        };
        //println!("{}", forum_ids.as_ref().unwrap().first().as_ref().unwrap());

        let mut conn = establish_connection();
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Migrations failed on database");

        State {
            email: var("email").expect("Required .env variable missing: email"),
            password: var("password").expect("Required .env variable missing: password"),
            client: client_builder
                .set_max_logging_length(99999999)
                .build(format!("https://{}:443/api/v1/api.php", website))
                .unwrap(),
            session_id: var("session_id").ok(),
            forum_ids: forum_ids,
            cafs: None,
            conn: conn,
        }
    }
}

fn establish_connection() -> SqliteConnection {
    let filename = var("database_file").expect("database_file must be set");
    SqliteConnection::establish(filename.as_str())
        .expect(format!("Error opening file {}", filename).as_str())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .expect("setting default subscriber failed");

    let mut state = State::new();

    if state.session_id.is_none() {
        let resp = state.client.login(&state.email, &state.password).await?;
        println!("{}", resp.session_id);
        state.session_id = Some(resp.session_id);
    }

    //If we can't get a session id by now, let's just exit the program
    state
        .session_id
        .as_ref()
        .expect("Can't get a valid session ID. Check your username and password.");

    if state.forum_ids.is_some() {
        get_forums(&mut state).await?;
    }
    else {
        println!("You didn't specify the environment variable `forum_ids`, so the tool is not going to extract anything from the forums. If this isn't what you intended, modify your .env file (or environment variable) for forum_ids according to the instructions in README.md.");
    }

    Ok(())
}
