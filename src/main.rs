use std::{collections::HashMap, thread, time::Duration};

use anyhow::bail;
use diesel::{*};
use jsonrpsee::{http_client::{HttpClientBuilder, HttpClient}, ws_client::HeaderMap, core::{Error, __reexports::serde::Deserialize, client::IdKind}, proc_macros::rpc};
use dotenvy::var;
use serde::Serialize;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

//SECTION: Diesel table generation

table! {
    subforums(forum_id) {
        title_welcome -> Nullable<Text>,
        preset_id -> Text,
        category_id -> Text,
        category_name -> Text,
        forum_id -> Text,
        forum_name -> Text,
        forum_description -> Text,
        parent_id -> Text,
        forum_type -> Text,
    }
}

table! {
    forum_posts(post_id) {
        post_id -> Text,
        post_time -> Text,
        post_content -> Text,
        post_user_id -> Text,
        last_edit_time -> Text,
        post_unhidden -> Text,
        post_admin_hidden -> Text,
        post_locked -> Text,
        last_edit_user -> Text,
        post_username -> Text,
        thread_id -> Nullable<Text>,
    }
}

table! {
    forum_threads(thread_id) {
        thread_id -> Text,
        thread_subject -> Text,
        thread_views -> Text,
        thread_type -> Text,
        thread_status -> Text,
        forum_id -> Text,
        username -> Nullable<Text>,
        category_id -> Text,
    }
}

table! {
    category_names(category_id) {
        category_id -> Text,
        category_name -> Text,
    }
}

table! {
    forum_presets(preset_id) {
        preset_id -> Text,
        title_welcome -> Text,
        total_threads -> Integer,
        total_posts -> Integer,
    }
}

table! {
    images(image_url) {
        image_url -> Text,
        image_content -> Binary,
    }
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    session_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ForumSettings {
    title_welcome: String,
}

#[derive(Serialize, Deserialize, Insertable)]
pub struct ForumThread {
    thread_id: String,
    thread_subject: String,
    thread_views: String,
    thread_type: String,
    thread_status: String,
    forum_id: String,
    username: Option<String>,
    category_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetForumResult {
    sticky: Vec<ForumThread>,
    threads: Vec<ForumThread>,
    announcement_local: Vec<ForumThread>,
    forum: Subforum,
    page: String,
    pages: u32
}

#[derive(Serialize, Deserialize)]
pub struct GetThreadResult {
    thread: ForumThread,
    posts: Vec<ForumPost>,
    total_items: String,
    pages: u32,
}

#[derive(Serialize, Deserialize, Insertable)]
pub struct ForumPost {
    post_id: String,
    post_time: String,
    post_content: String,
    post_user_id: String,
    last_edit_time: String,
    post_unhidden: String,
    post_admin_hidden: String,
    post_locked: String,
    last_edit_user: String,
    post_username: String,
    thread_id: Option<String>,
}

#[derive(Serialize, Deserialize, Insertable)]
pub struct Subforum {
    title_welcome: Option<String>,
    preset_id: String,
    category_id: String,
    category_name: String,
    forum_id: String,
    forum_name: String,
    forum_description: String,
    parent_id: String,
    forum_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetCafResult {
    settings: ForumSettings,
    subforums: HashMap<String, Vec<Subforum>>,
    total_threads: u32,
    total_posts: u32,
    category_names: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Insertable, Queryable)]
pub struct Image {
    image_url: String,
    image_content: Vec<u8>
}

#[rpc(client)]
trait Api {
    #[method(name="User.login", param_kind=map)]
    async fn login(&self, email: &String, password: &String) -> Result<LoginResponse, Error>;

    #[method(name="Forum.getCategoriesAndForums", param_kind=map)]
    async fn get_categories_and_forums(&self, session_id: &String, preset_id: &String) -> Result<GetCafResult, Error>;

    #[method(name="Forum.getForum", param_kind=map)]
    async fn get_forum(&self, session_id: &String, forum_id: &String, page: Option<&String>) -> Result<GetForumResult, Error>;

    #[method(name="Forum.getThread", param_kind=map)]
    async fn get_thread(&self, session_id: &String, thread_id: &String, page: Option<&String>) -> Result<GetThreadResult, Error>;
}

struct State {
    email: String,
    password: String,
    client: HttpClient,
    session_id: Option<String>,
    forum_ids: Option<Vec<String>>,
    cafs: Option<Vec<GetCafResult>>,
    conn: SqliteConnection,
}

impl State {
    fn new() -> Self {        
        let proxy = var("proxy").ok();
        let website = var("website").expect("Required .env variable missing: website");
        let mut headers = HeaderMap::new();
        headers.insert("Accept", "*/*".parse().unwrap());
        headers.insert("User-Agent", "encuum-api".parse().unwrap());
        let mut client_builder = HttpClientBuilder::default().set_headers(headers).id_format(IdKind::String);
        if proxy.as_ref().is_some() {
            client_builder = client_builder.set_proxy(proxy.as_ref().unwrap()).unwrap();
        }

        let forum_preset_ids = var("forum_ids").ok();
        let forum_ids: Option<Vec<String>> = match forum_preset_ids {
            Some(fpis) => Some(fpis.split(",").map(|s| s.to_string()).collect()),
            None => None
        };
        //println!("{}", forum_ids.as_ref().unwrap().first().as_ref().unwrap());

        let mut conn = establish_connection();
        conn.run_pending_migrations(MIGRATIONS).expect("Migrations failed on database");
        
        State {
            email: var("email").expect("Required .env variable missing: email"),
            password: var("password").expect("Required .env variable missing: password"),
            client: client_builder.set_max_logging_length(99999999).build(format!("https://{}:443/api/v1/api.php", website)).unwrap(),
            session_id: var("session_id").ok(),
            forum_ids: forum_ids,
            cafs: None,
            conn: conn
        }
    }
}

async fn get_forums(state: &mut State) -> anyhow::Result<()> {
    let mut cafs: Vec<GetCafResult> = vec![];
    let mut threads: Vec<ForumThread> = vec![];
    let mut categories: &HashMap<String, String>;
    let conn = &mut state.conn;

    for caf_id in state.forum_ids.as_ref().unwrap() {
        let mut caf: GetCafResult;

        //XXX: Figure out how to write a closure that does this so I don't have to copy paste this 3 times
        let mut ctries = 0;
        'cafgeez: loop {
            let caff = state.client.get_categories_and_forums(state.session_id.as_ref().unwrap(), &caf_id).await;
            let my_err: Error;
            match caff {
                Ok(c) => { 
                    caf = c; 
                    break 'cafgeez;
                },
                Err(e) => {
                    println!("{}", e);
                    my_err = e;
                }
            }

            if ctries >= 5 {
                bail!(my_err);
            }

            ctries += 1;
            thread::sleep(Duration::from_secs(60));
        }
        
        println!("got a site forum instance (aka prefix or caf) {} called {}", caf_id, &caf.settings.title_welcome);
        categories = &caf.category_names;

        for (cid, cn) in categories {
            diesel::insert_or_ignore_into(category_names::table)
                .values((category_names::category_id.eq(cid), category_names::category_name.eq(cn)))
                .execute(conn)
                .expect("Error saving new subforum");
        }

        for (_forum_id, subforums) in caf.subforums.iter_mut() {
            for subforum in subforums {
                let mut forum_curr_page: u32 = 1;
                let mut forum_pages: u32;

                //Loop through each page of the subforum thread index.
                loop {
                    let mut sf: GetForumResult;

                    let mut gtries = 0;
                    'gfrgeez: loop {
                        let sff = state.client.get_forum(state.session_id.as_ref().unwrap(), &subforum.forum_id, Some(&forum_curr_page.to_string())).await;
                        let my_err: Error;
                        match sff {
                            Ok(c) => { 
                                sf = c; 
                                break 'gfrgeez;
                            },
                            Err(e) => {
                                println!("{}", e);
                                my_err = e;
                            }
                        }

                        if gtries >= 5 {
                            bail!(my_err);
                        }

                        gtries += 1;
                        thread::sleep(Duration::from_secs(60));
                    }
                    
                    println!("got page {}/{} of subforum {} called {}", sf.page, sf.pages, sf.forum.forum_id, sf.forum.forum_name);
                    if forum_curr_page == 1 {
                        diesel::insert_or_ignore_into(subforums::table)
                            .values(&sf.forum)
                            .execute(conn)
                            .expect("Error saving new subforum");
                    }
                    forum_pages = sf.pages;
                    for thread in sf.threads.iter_mut() {
                        let mut thread_curr_page: u32 = 1;
                        let mut thread_pages: u32;

                        //Loop through each post of a thread.
                        loop {
                            let mut gtr: GetThreadResult;
                            
                            let mut ttries = 0;
                            'gtrgeez: loop {
                                let gtrr = state.client.get_thread(state.session_id.as_ref().unwrap(), &thread.thread_id, Some(&thread_curr_page.to_string())).await;
                                let my_err: Error;
                                match gtrr {
                                    Ok(c) => { 
                                        gtr = c; 
                                        break 'gtrgeez;
                                    },
                                    Err(e) => {
                                        println!("{}", e);
                                        my_err = e;
                                    }
                                }

                                if ttries >= 5 {
                                    bail!(my_err);
                                }

                                ttries += 1;
                                thread::sleep(Duration::from_secs(60));
                            }
                            
                            
                            thread_pages = gtr.pages;
                            println!("got page {}/{} of a thread {} called {}", thread_curr_page, thread_pages, gtr.thread.thread_id, gtr.thread.thread_subject);
                            for post in gtr.posts.iter_mut() {
                                println!("got a post {} related to thread {}", post.post_id, thread.thread_id);
                                post.thread_id = Some(thread.thread_id.clone());
                            }
                            
                            diesel::insert_or_ignore_into(forum_posts::table)
                                .values(&gtr.posts)
                                .execute(conn)
                                .expect("Error saving new posts");

                            //XXX: This MUST be done AFTER the diesel call, because .append REMOVES from the source vec! 
                            //posts.append(&mut gtr.posts);

                            if thread_curr_page >= thread_pages {
                                break;
                            }

                            thread_curr_page += 1;
                        }
                    }

                    for thread_vec in vec![&sf.threads, &sf.sticky, &sf.announcement_local] {
                        diesel::insert_or_ignore_into(forum_threads::table)
                            .values(thread_vec)
                            .execute(conn)
                            .expect("Error saving new threads");
                    }

                    //XXX: This MUST be done AFTER the diesel call, because .append REMOVES from the source vec! 
                    threads.append(&mut sf.threads);
                    threads.append(&mut sf.sticky);
                    threads.append(&mut sf.announcement_local);

                    if forum_curr_page >= forum_pages {
                        break; //Exit the forum thread index loop
                    }

                    forum_curr_page += 1;
                }
            }
        }
        cafs.push(caf);
    }



    state.cafs = Some(cafs);
    Ok(())
}

fn establish_connection() -> SqliteConnection {
    let filename = var("database_file").expect("database_file must be set");
    SqliteConnection::establish(filename.as_str()).expect(format!("Error opening file {}", filename).as_str())
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
    state.session_id.as_ref().expect("Can't get a valid session ID. Check your username and password.");
    
    if state.forum_ids.is_some() {
        get_forums(&mut state).await?;
    }
    
    Ok(())
}
