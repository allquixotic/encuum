/// Copyright (c) 2023, Sean McNamara <smcnam@gmail.com>.
/// All code in this repository is disjunctively licensed under [CC-BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/) and [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0).
/// Direct dependencies are believed to be under a license which allows downstream code to have these licenses.
pub mod forum;
pub mod structures;


use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::time::Duration;

use crate::forum::*;
use crate::structures::*;
use dotenvy::var;
use lazy_static::lazy_static;
use migration::Migrator;
use migration::MigratorTrait;

use sea_orm::Database;
use secrecy::ExposeSecret;
use secrecy::SecretString;
use tokio_cron_scheduler::Job;
use tokio_cron_scheduler::JobScheduler;
use tracing::Level;
use tracing::info;
use tracing::warn;

impl State {
    pub async fn new() -> Self {
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

        let session_id = match var("session_id").ok() {
            Some(s) => Some(SecretString::new(s)),
            None => None,
        };

        State {
            email: var("email").expect("Required .env variable missing: email"),
            password: SecretString::new(
                var("password").expect("Required .env variable missing: password"),
            ),
            session_id: session_id,
            forum_ids: forum_ids,
            cafs: None,
            subforum_ids: subforum_ids,
            keep_going: var("keep_going")
                .unwrap_or("false".to_string())
                .parse()
                .unwrap(),
            do_images: var("do_images")
                .unwrap_or("true".to_string())
                .parse()
                .unwrap(),
            sanitize_log: var("sanitize_log")
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

struct MultiWriter {
    writers: Vec<Box<dyn Write + Send + Sync>>,
}

impl Write for MultiWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for writer in self.writers.iter_mut() {
            writer.write(buf)?;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        for writer in self.writers.iter_mut() {
            writer.flush()?;
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let mut writers: Vec<Box<dyn Write + Send + Sync>> = vec![(Box::new(std::io::stderr()))];
    if let Some(log_file) = var("log_file").ok() {
        writers.push(Box::new(BufWriter::new(File::create(log_file).unwrap())));
    }
    let mw = Mutex::new(MultiWriter { writers });

    let tsb = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).with_ansi(false)
        .with_writer(mw);

    if let Ok(log_level) = var("log_level") {
        match log_level.to_uppercase().as_str() {
            "TRACE" => tsb.with_max_level(Level::TRACE),
            "DEBUG" => tsb.with_max_level(Level::DEBUG),
            "INFO" => tsb.with_max_level(Level::INFO),
            "WARN" => tsb.with_max_level(Level::WARN),
            "ERROR" => tsb.with_max_level(Level::ERROR),
            _ => tsb.with_max_level(Level::INFO)
        }
        .try_init().expect("setting default subscriber failed");
    }   

    let sched = JobScheduler::new().await?;

    let stats_job =
        Job::new_repeated(
            Duration::from_secs(60),
            |_a, _schedd| match memory_stats::memory_stats() {
                Some(ms) => {
                    info!(
                        "*** encuum memory usage: {} bytes ({} MB)",
                        ms.physical_mem,
                        ms.physical_mem / 1000000
                    );
                }
                None => {
                    warn!("*** unable to get encuum memory usage");
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
                    let resp = SEE
                        .login(&state.email, &state.password.expose_secret())
                        .await
                        .expect("FATAL ERROR: Login failed");
                    if !state.sanitize_log {
                        info!("Your session ID is: {}", resp.session_id);
                    }
                    state.session_id = Some(SecretString::new(resp.session_id));
                }

                //If we can't get a session id by now, let's just exit the program
                state
                    .session_id
                    .as_ref()
                    .expect("FATAL ERROR: Can't get a valid session ID. Check your username and password.");

                if state.forum_ids.is_some() {
                    let fd = ForumDoer { state: state };
                    fd.get_forums().await.unwrap();
                } else {
                    warn!("You didn't specify the environment variable `forum_ids`, so the tool is not going to extract anything from the forums. If this isn't what you intended, modify your .env file (or environment variable) for forum_ids according to the instructions in README.md.");
                }
                STOPPIT.store(true, Ordering::Relaxed);
                info!("*** Stopping tasks...");
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
            info!("Encuum exited normally.");
            break;
        }
    }
    Ok(())
}
