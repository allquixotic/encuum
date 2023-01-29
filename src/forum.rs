/// Copyright (c) 2023, Sean McNamara <smcnam@gmail.com>.
/// All code in this repository is disjunctively licensed under [CC-BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/) and [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0).
/// Direct dependencies are believed to be under a license which allows downstream code to have these licenses.
use entity::*;

use crate::structures::*;
use sea_orm::{sea_query::OnConflict, EntityTrait, Set};

use futures::{stream::FuturesUnordered, StreamExt};
use jsonrpsee::core::Error;
use jsonrpsee::proc_macros::rpc;
use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::HashMap, time::Duration};

#[rpc(client)]
trait ForumApi {
    #[method(name="User.login", param_kind=map)]
    async fn login(&self, email: &String, password: &String) -> Result<LoginResponse, Error>;

    #[method(name="Forum.getCategoriesAndForums", param_kind=map)]
    async fn get_categories_and_forums(
        &self,
        session_id: &String,
        preset_id: &String,
    ) -> Result<GetCafResult, Error>;

    #[method(name="Forum.getForum", param_kind=map)]
    async fn get_forum(
        &self,
        session_id: &String,
        forum_id: &String,
        page: Option<&String>,
    ) -> Result<GetForumResult, Error>;

    #[method(name="Forum.getThread", param_kind=map)]
    async fn get_thread(
        &self,
        session_id: &String,
        thread_id: &String,
        page: Option<&String>,
    ) -> Result<GetThreadResult, Error>;
}

lazy_static! {
    pub static ref IMG_RX: Regex = Regex::new(r"(?i)\[img]\s*(https?://.+?)\s*\[/img]").unwrap();
}

#[derive(Debug)]
pub enum Thing {
    Preset,
    ForumIndex,
    Thread,
}

pub struct ForumDoer {
    pub state: State,
}

pub fn parse_number(val: &serde_json::Value) -> Option<u32> {
    match val {
        sea_orm::JsonValue::Null => None,
        sea_orm::JsonValue::Bool(_) => panic!("Expected number, got bool"),
        sea_orm::JsonValue::Number(n) => Some(n.as_u64().unwrap().try_into().unwrap()),
        sea_orm::JsonValue::String(s) => Some(s.parse::<u32>().unwrap()),
        sea_orm::JsonValue::Array(_) => panic!("Expected number, got array"),
        sea_orm::JsonValue::Object(_) => panic!("Expected number, got object"),
    }
}

//Slow down the calls ever so slightly to reduce the chance of being rate-limited
pub async fn whoa(arl: &mut u32) {
    tokio::time::sleep(Duration::from_millis((100 * *arl).into())).await;
    if *arl < 5 {
        *arl += 1;
    }
}

pub async fn calculate_and_sleep(thing: &Thing, thing_id: &String, e: &Error, tries: &u32) {
    let mut dur: u32 = 30;
    if e.to_string().contains("status code: 429") {
        dur = 30 + (60 * tries * tries); // 30 + 60x^2 quadratic backoff
        println!("For {:?} {}: HTTP response code 429 means Enjin rate-limited us for going too fast! Waiting {} seconds.",
        thing, thing_id, dur);
    }
    tokio::time::sleep(Duration::from_secs(dur.into())).await;
}

impl ForumDoer {
    /// Since image downloads are unreliable anyway, we just print out errors and keep going
    pub async fn download_image(&self, url: String) {
        println!("download_image({:?})", url);
        let db_find = images::Entity::find_by_id(url.to_string())
            .one(&self.state.conn)
            .await;

        if db_find.is_err() {
            dbg!(db_find.unwrap_err());
            return;
        }

        let db_result = db_find.unwrap();
        if db_result.is_some() {
            println!("Already have image; not downloading again: {}", url);
            return;
        }

        let req = self.state.req_client.get(&url).send().await;
        if req.is_err() {
            dbg!(req.unwrap_err());
            return;
        }
        let maybe_bytes = req.unwrap().bytes().await;
        if maybe_bytes.is_err() {
            dbg!(maybe_bytes.unwrap_err());
            return;
        }
        let bytes = maybe_bytes.unwrap();
        let maybe_exec = images::Entity::insert(images::ActiveModel {
            image_url: Set(url),
            image_content: Set(Some(bytes.to_vec())),
        })
        .exec(&self.state.conn)
        .await;
        if maybe_exec.is_err() {
            dbg!(maybe_exec.unwrap_err());
        }
    }

    pub async fn get_images(&self, post_id: String, post_content: String) {
        println!("get_images({:?})", post_id);
        let matches = IMG_RX.captures_iter(&post_content);

        for mmatch in matches {
            let url = &mmatch[1];
            self.download_image(url.to_owned()).await;
        }
    }

    pub async fn get_preset_retry(&self, preset_id: &String) -> Option<GetCafResult> {
        println!("get_preset_retry({:?})", preset_id);
        let mut tries = 1;

        loop {
            let maybe_caf = self
                .state
                .client
                .get_categories_and_forums(&self.state.session_id.as_ref().unwrap(), preset_id)
                .await;
            match maybe_caf {
                Err(e) => {
                    let f = format!("Preset: {}, Try #{}: {}", preset_id, tries, e);
                    if tries >= 5 {
                        if self.state.keep_going {
                            return None;
                        } else {
                            panic!("{}", f);
                        }
                    }
                    dbg!(f);
                    tries += 1;
                    calculate_and_sleep(&Thing::Preset, preset_id, &e, &tries).await;
                }
                Ok(caf) => {
                    println!(
                        "got a site forum instance (aka prefix or caf) {} called {}",
                        preset_id, &caf.settings.title_welcome
                    );
                    return Some(caf);
                }
            }
        }
    }

    pub async fn get_forum_index_retry(
        &self,
        forum_id: &String,
        page: Option<String>,
    ) -> Option<GetForumResult> {
        println!("get_forum_index_retry({:?}, {:?})",forum_id, page);
        let mut tries = 1;
        loop {
            let maybe_gfr = self
                .state
                .client
                .get_forum(
                    &self.state.session_id.as_ref().unwrap(),
                    &forum_id,
                    page.as_ref(),
                )
                .await;
            if maybe_gfr.is_err() {
                let e = maybe_gfr.unwrap_err();
                let f = format!("Subforum: {}, Try #{}: {}", forum_id, tries, e);
                if tries >= 5 {
                    if self.state.keep_going {
                        return None;
                    } else {
                        panic!("{}", f);
                    }
                }
                dbg!(f);
                tries += 1;
                calculate_and_sleep(&Thing::ForumIndex, forum_id, &e, &tries).await;
            } else {
                return maybe_gfr.ok();
            }
        }
    }

    pub async fn get_thread_index_retry(
        &self,
        thread_id: &String,
        page: Option<String>,
    ) -> Option<GetThreadResult> {
        println!("get_thread_index_retry({}, {:?})",thread_id, page);
        let mut tries = 1;
        loop {
            let maybe_gtr = self
                .state
                .client
                .get_thread(
                    &self.state.session_id.as_ref().unwrap(),
                    thread_id,
                    page.as_ref(),
                )
                .await;
            match maybe_gtr {
                Err(e) => {
                    let f = format!("Thread: {}, Try #{}: {}", thread_id, tries, e);
                    if tries >= 5 {
                        if self.state.keep_going {
                            return None;
                        } else {
                            panic!("{}", f);
                        }
                    }
                    dbg!(f);
                    tries += 1;
                    calculate_and_sleep(&Thing::Thread, thread_id, &e, &tries).await;
                }
                Ok(gtr) => {
                    return Some(gtr);
                }
            }
        }
    }

    pub async fn get_subforums(&self, subforum_ids: Vec<&String>) -> Vec<GetForumResult> {
        println!("get_subforums({:#?})", subforum_ids);
        let mut retval: Vec<GetForumResult> = vec![];
        let mut page_map: HashMap<String, u32> = HashMap::new();
        let mut futs = FuturesUnordered::new();

        //Queue request for first page for every subforum index list.
        for forum_id in subforum_ids {
            futs.push(self.get_forum_index_retry(forum_id, None));
        }

        let mut arl: u32 = 1;
        while let Some(x) = futs.next().await {
            //This is true when we have keep_going on and the forum index couldn't be retrieved.
            if x.is_none() {
                continue;
            }
            let y = x.unwrap(); //Guaranteed to succeed.
            let page_num = parse_number(&y.pages);
            if let Some(pn) = page_num {
                page_map.insert(y.forum.forum_id.clone(), pn);
            }
            println!(
                "Got Page 1 of the Forum Index for {}",
                y.forum.forum_id
            );
            whoa(&mut arl).await;
            retval.push(y);
        }

        //Queue request for every remaining page for every subforum index list.
        for (forum_id, pages) in page_map.iter_mut() {
            let mut page = 2;
            while page <= *pages {
                futs.push(self.get_forum_index_retry(forum_id, Some(page.to_string())));
                page += 1;
            }
        }

        //Fill out the list of forum indexes from all the pages.
        arl = 1;
        while let Some(x) = futs.next().await {
            if x.is_none() {
                continue;
            }
            let xu = x.unwrap();
            println!(
                "Got Page {:?}/{:?} for Forum {} from Preset {}",
                xu.page, xu.pages, xu.forum.forum_id, xu.forum.preset_id
            );
            retval.push(xu);
            whoa(&mut arl).await;
        }

        return retval;
    }

    pub async fn get_threads(&self, mut thread_ids: Vec<String>) -> Vec<GetThreadResult> {
        println!("get_threads({:#?})", thread_ids);
        let mut retval = vec![];
        let mut page_map: HashMap<String, u32> = HashMap::new();
        let mut futs = FuturesUnordered::new();

        //Queue request for first page for every thread index list.
        for thread_id in thread_ids.iter_mut() {
            futs.push(self.get_thread_index_retry(thread_id, None));
        }

        let mut arl: u32 = 1;
        while let Some(x) = futs.next().await {
            //This is true when we have keep_going on and the thread index couldn't be retrieved.
            if x.is_none() {
                continue;
            }
            let y = x.unwrap(); //Guaranteed to succeed.
            let page_num = parse_number(&y.pages);
            if let Some(pn) = page_num {
                page_map.insert(y.thread.thread_id.clone(), pn);
            }
            whoa(&mut arl).await;
            println!("Got Page 1 of Thread {} from Forum {}",y.thread.thread_id, &y.thread.forum_id);
            retval.push(y);
        }

        //Queue request for every remaining page for every thread index list.
        for (thread_id, pages) in page_map.iter_mut() {
            let mut page = 2;
            while page <= *pages {
                futs.push(self.get_thread_index_retry(thread_id, Some(page.to_string())));
                page += 1;
            }
        }

        //Fill out the list of thread indexes from all the pages.
        arl = 1;
        while let Some(x) = futs.next().await {
            if x.is_none() {
                continue;
            }
            let xu = x.unwrap();
            println!("Got Thread {} from Forum {}",xu.thread.thread_id, xu.thread.forum_id);
            retval.push(xu);
            whoa(&mut arl).await;
        }

        return retval;
    }

    pub async fn save_preset(&self, preset_id: &String, caf: &GetCafResult) {
        println!("save_preset({})", preset_id);
        let categories = &caf.category_names;

        for (cid, cn) in categories {
            category_names::Entity::insert(category_names::ActiveModel {
                category_id: Set(cid.to_string()),
                category_name: Set(cn.to_string()),
            })
            .on_conflict(
                // on conflict do nothing
                OnConflict::column(category_names::Column::CategoryId)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(&self.state.conn)
            .await
            .expect("Code-up error: can't insert a category name into the database!");
        }

        forum_presets::Entity::insert(forum_presets::ActiveModel {
            preset_id: Set(preset_id.to_string()),
            title_welcome: Set(caf.settings.title_welcome.clone()),
            total_threads: Set(parse_number(&caf.total_threads)
                .unwrap()
                .try_into()
                .unwrap()),
            total_posts: Set(parse_number(&caf.total_posts).unwrap().try_into().unwrap()),
        })
        .on_conflict(
            // on conflict do nothing
            OnConflict::column(forum_presets::Column::PresetId)
                .update_columns([
                    forum_presets::Column::TitleWelcome,
                    forum_presets::Column::TotalPosts,
                    forum_presets::Column::TotalThreads,
                ])
                .to_owned(),
        )
        .exec(&self.state.conn)
        .await
        .expect("Code-up error: can't insert a forum preset into the database!");
    }

    pub async fn save_subforum(&self, gfr: &GetForumResult) {
        println!("save_subforum({})", gfr.forum.forum_id);
        let mut futs = FuturesUnordered::new();

        subforums::Entity::insert(subforums::ActiveModel {
            title_welcome: Set(gfr.forum.title_welcome.clone()),
            preset_id: Set(gfr.forum.preset_id.clone()),
            category_id: Set(gfr.forum.category_id.clone()),
            category_name: Set(gfr.forum.category_name.clone()),
            forum_id: Set(gfr.forum.forum_id.clone()),
            forum_name: Set(gfr.forum.forum_name.clone()),
            forum_description: Set(gfr.forum.forum_description.clone()),
            parent_id: Set(gfr.forum.parent_id.clone()),
            forum_type: Set(gfr.forum.forum_type.clone()),
        })
        .on_conflict(
            // on conflict do nothing
            OnConflict::column(subforums::Column::ForumId)
                .update_columns([
                    subforums::Column::TitleWelcome,
                    subforums::Column::PresetId,
                    subforums::Column::CategoryId,
                    subforums::Column::CategoryName,
                    subforums::Column::ForumName,
                    subforums::Column::ForumDescription,
                    subforums::Column::ParentId,
                    subforums::Column::ForumType,
                ])
                .to_owned(),
        )
        .exec(&self.state.conn)
        .await
        .expect("Code-up error: can't insert a subforum into the database!");

        for thread in &gfr.threads {
            futs.push(
                forum_threads::Entity::insert(forum_threads::ActiveModel {
                    thread_id: Set(thread.thread_id.clone()),
                    thread_subject: Set(thread.thread_subject.clone()),
                    thread_views: Set(thread.thread_views.clone()),
                    thread_type: Set(thread.thread_type.clone()),
                    thread_status: Set(thread.thread_status.clone()),
                    forum_id: Set(gfr.forum.forum_id.clone()),
                    username: Set(thread.username.clone()),
                    category_id: Set(gfr.forum.category_id.clone()),
                })
                .on_conflict(
                    // on conflict do nothing
                    OnConflict::column(forum_threads::Column::ThreadId)
                        .update_columns([
                            forum_threads::Column::ThreadSubject,
                            forum_threads::Column::ThreadViews,
                            forum_threads::Column::ThreadType,
                            forum_threads::Column::ThreadStatus,
                            forum_threads::Column::ForumId,
                            forum_threads::Column::Username,
                            forum_threads::Column::CategoryId,
                        ])
                        .to_owned(),
                )
                .exec(&self.state.conn),
            );
        }
        while let Some(x) = futs.next().await {
            x.expect("Code-up error: can't insert a forum thread into the database!");
        }
    }

    pub async fn save_threads(&self, gtrs: Vec<GetThreadResult>) {
        println!("save_threads()");
        let mut futs = FuturesUnordered::new();
        let mut img_futs = FuturesUnordered::new();
        for gtr in gtrs {
            for post in &gtr.posts {
                futs.push(
                    forum_posts::Entity::insert(forum_posts::ActiveModel {
                        post_id: Set(post.post_id.clone()),
                        post_time: Set(post.post_time.clone()),
                        post_content: Set(post.post_content.clone()),
                        post_user_id: Set(post.post_user_id.clone()),
                        last_edit_time: Set(post.last_edit_time.clone()),
                        post_unhidden: Set(post.post_unhidden.clone()),
                        post_admin_hidden: Set(post.post_admin_hidden.clone()),
                        post_locked: Set(post.post_locked.clone()),
                        last_edit_user: Set(post.last_edit_user.clone()),
                        post_username: Set(post.post_username.clone()),
                        thread_id: Set(Some(gtr.thread.thread_id.clone())),
                    })
                    .on_conflict(
                        // on conflict do nothing
                        OnConflict::column(forum_posts::Column::PostId)
                            .update_columns([
                                forum_posts::Column::PostTime,
                                forum_posts::Column::PostContent,
                                forum_posts::Column::PostUserId,
                                forum_posts::Column::LastEditTime,
                                forum_posts::Column::PostUnhidden,
                                forum_posts::Column::PostAdminHidden,
                                forum_posts::Column::PostLocked,
                                forum_posts::Column::LastEditUser,
                                forum_posts::Column::PostUsername,
                                forum_posts::Column::ThreadId,
                            ])
                            .to_owned(),
                    )
                    .exec(&self.state.conn),
                );

                img_futs.push(self.get_images(post.post_id.clone(), post.post_content.clone()));
            }
        }
        while let Some(x) = futs.next().await {
            x.expect("Code-up error: can't insert a forum post into the database!");
        }

        while let Some(_x) = img_futs.next().await {}
    }

    pub async fn get_forums(&self) -> anyhow::Result<()> {
        for caf_id in self.state.forum_ids.as_ref().unwrap() {
            let subforum_results;
            let mut db_futs = FuturesUnordered::new();
            let maybe_caf = self.get_preset_retry(caf_id).await;
            if maybe_caf.is_none() {
                continue;
            }

            let caf = maybe_caf.unwrap(); //Guaranteed to succeed
            self.save_preset(caf_id, &caf).await;
            let maybe_sfis = &self.state.subforum_ids;
            let mut all_subforums: Vec<String> = vec![];
            all_subforums.extend(caf.subforums.keys().cloned());  

            //Add the "categories" top-level forums.
            for foru in caf.categories.values() {
                for (fid, _) in foru {
                    all_subforums.push(fid.clone());
                }
            }

            //Add all the subforums.
            for sfs in caf.subforums.values() {
                for sf in sfs {
                    all_subforums.push(sf.forum_id.clone());
                }
            }

            all_subforums.sort();
            all_subforums.dedup();

            //Call Forum.getForum for every CAF (only for allowed subforums).
            let allowed_subforums =
                Vec::from_iter(all_subforums.iter().filter(|subforum_id| match maybe_sfis {
                    Some(sfis) => {
                        if sfis.len() == 0 {
                            return true;
                        } else {
                            return sfis.contains(subforum_id);
                        }
                    }
                    None => {
                        return true;
                    }
                }));
            println!("*** Total number of forums and subforums to scan: {}", allowed_subforums.len());
            subforum_results = self.get_subforums(allowed_subforums).await;
            let mut gtr_futs = FuturesUnordered::new();
            //Call Forum.getThread for every GFR.
            for gfr in &subforum_results {
                db_futs.push(self.save_subforum(&gfr));
                let mut inval = vec![];
                for inv in &gfr.threads {
                    inval.push(inv.thread_id.clone());
                }
                let threads = self.get_threads(inval).await;
                gtr_futs.push(self.save_threads(threads));
            }

            while let Some(_) = db_futs.next().await {}
            while let Some(_) = gtr_futs.next().await {}
        }
        Ok(())
    }
}
