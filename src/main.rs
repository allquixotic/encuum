/// Copyright (c) 2023, Sean McNamara <smcnam@gmail.com>.
/// All code in this repository is disjunctively licensed under [CC-BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/) and [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0).
/// Direct dependencies are believed to be under a license which allows downstream code to have these licenses.
pub mod forum;
pub mod structures;

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;

use crate::forum::*;
use crate::structures::*;
use dotenvy::var;
use jsonrpsee::{core::client::IdKind, http_client::HttpClientBuilder, ws_client::HeaderMap};
use lazy_static::lazy_static;
use migration::Migrator;
use migration::MigratorTrait;
use sea_orm::Database;
use tokio_cron_scheduler::Job;
use tokio_cron_scheduler::JobScheduler;

impl State {
    pub async fn new() -> Self {
        let proxy = var("proxy").ok();
        let website = var("website").expect("Required .env variable missing: website");
        let mut headers = HeaderMap::new();
        headers.insert("Accept", "*/*".parse().unwrap());
        headers.insert("User-Agent", "encuum-api".parse().unwrap());
        let mut client_builder = HttpClientBuilder::default()
            .set_headers(headers)
            .id_format(IdKind::String)
            .request_timeout(Duration::from_secs(600));
        if proxy.as_ref().is_some() {
            client_builder = client_builder.set_proxy(proxy.as_ref().unwrap()).unwrap();
        }

        let forum_preset_ids = var("forum_ids").ok();
        let forum_ids: Option<Vec<String>> = match forum_preset_ids {
            Some(fpis) => Some(fpis.split(",").map(|s| s.to_string()).collect()),
            None => None,
        };

        let subforum_ids_opt = var("subforum_ids").ok();
        let subforum_ids: Option<Vec<String>> = match subforum_ids_opt {
            Some(fpis) => Some(fpis.split(",").map(|s| s.to_string()).collect()),
            None => None,
        };

        let filename = var("database_file").expect("database_file must be set");
        let conn = Database::connect(format!("sqlite://./{}?mode=rwc", filename))
            .await
            .expect(format!("Can't open DB {}", filename).as_str());
        Migrator::up(&conn, None)
            .await
            .expect("Failed to bring DB schema up");

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
            subforum_ids: subforum_ids,
            keep_going: var("keep_going")
                .unwrap_or("false".to_string())
                .parse()
                .unwrap(),
            req_client: reqwest::Client::new(),
            conn: conn,
        }
    }
}

lazy_static! {
    static ref STOPPIT: AtomicBool = AtomicBool::new(false);
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init()
        .expect("setting default subscriber failed");

    let sched = JobScheduler::new().await?;

    let stats_job =
        Job::new_repeated(
            Duration::from_secs(60),
            |_a, _schedd| match memory_stats::memory_stats() {
                Some(ms) => {
                    println!(
                        "*** encuum memory usage: {} bytes ({} MB)",
                        ms.physical_mem,
                        ms.physical_mem / 1000000
                    );
                }
                None => {
                    println!("*** unable to get encuum memory usage");
                }
            },
        )?;

    sched.add(stats_job).await?;

    let oneshot = Job::new_one_shot_at_instant_async(
        std::time::Instant::now(),
        |_a, mut schedd| {
            Box::pin(async move {
                let mut state = State::new().await;
                if state.session_id.is_none() {
                    let resp = state
                        .client
                        .login(&state.email, &state.password)
                        .await
                        .expect("Login failed");
                    println!("{}", resp.session_id);
                    state.session_id = Some(resp.session_id);
                }

                //If we can't get a session id by now, let's just exit the program
                state
                    .session_id
                    .as_ref()
                    .expect("Can't get a valid session ID. Check your username and password.");

                if state.forum_ids.is_some() {
                    let fd = ForumDoer { state: state };
                    fd.get_forums().await.unwrap();
                } else {
                    println!("You didn't specify the environment variable `forum_ids`, so the tool is not going to extract anything from the forums. If this isn't what you intended, modify your .env file (or environment variable) for forum_ids according to the instructions in README.md.");
                }
                STOPPIT.store(true, Ordering::Relaxed);
                println!("*** Stopping tasks...");
                schedd.shutdown().await.unwrap();
            })
        },
    )?;
    sched.add(oneshot).await?;

    sched.start().await?;

    // Wait a while so that the jobs actually run
    loop {
        tokio::time::sleep(core::time::Duration::from_secs(10)).await;
        if STOPPIT.load(Ordering::Relaxed) {
            println!("Exiting.");
            break;
        }
    }
    Ok(())
}
