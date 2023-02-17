use crate::dumbsert;
use crate::exposed_session;
use crate::helpers::*;
/// Copyright (c) 2023, Sean McNamara <smcnam@gmail.com>.
/// All code in this repository is disjunctively licensed under [CC-BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/) and [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0).
/// Direct dependencies are believed to be under a license which allows downstream code to have these licenses.
use crate::state;
use crate::structures::*;
use entity::*;
use futures::{stream::FuturesUnordered, StreamExt};
use jsonrpsee::proc_macros::rpc;
use lazy_static::lazy_static;
use regex::Regex;
use sea_orm::{sea_query::OnConflict, EntityTrait, Set};
use secrecy::ExposeSecret;
use std::collections::{HashMap, HashSet};
use std::iter::*;
use tracing::{debug, info, warn};

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

/// Since image downloads are unreliable anyway, we just print out errors and keep going
pub async fn download_image(url: String) {
    debug!("download_image({:?})", url);

    let db_find = images::Entity::find_by_id(url.to_string())
        .one(&state!().conn)
        .await;

    if db_find.is_err() {
        warn!("{:#?}", db_find.unwrap_err());
        return;
    }

    let db_result = db_find.unwrap();
    if db_result.is_some() {
        info!("Already have image; not downloading again: {}", url);
        return;
    }
    let req = state!().req_client.get(&url).send().await;
    if req.is_err() {
        warn!("{:#?}", req.unwrap_err());
        return;
    }
    let maybe_bytes = req.unwrap().bytes().await;
    if maybe_bytes.is_err() {
        warn!("{:#?}", maybe_bytes.unwrap_err());
        return;
    }
    let bytes = maybe_bytes.unwrap();
    let maybe_exec = images::Entity::insert(images::ActiveModel {
        image_url: Set(url.clone()),
        image_content: Set(Some(bytes.to_vec())),
    })
    .exec(&state!().conn)
    .await;
    if maybe_exec.is_err() {
        warn!("{:#?}", maybe_exec.unwrap_err());
    }
}

pub async fn get_images(post_id: String, post_content: String) {
    debug!("get_images({:?})", post_id);
    let matches = IMG_RX.captures_iter(&post_content);

    for mmatch in matches {
        let url = &mmatch[1];
        download_image(url.to_owned()).await;
    }
}

pub async fn get_preset_retry(preset_id: &String) -> Option<GetCafResult> {
    debug!("get_preset_retry({:?})", preset_id);
    let mut tries = 1;

    loop {
        let maybe_caf = SEE
            .get_categories_and_forums(exposed_session!(), preset_id)
            .await;
        match maybe_caf {
            Err(e) => {
                let f = format!("Preset: {}, Try #{}: {}", preset_id, tries, e);
                if tries >= 5 {
                    if state!().keep_going {
                        return None;
                    } else {
                        panic!("{}", f);
                    }
                }
                warn!("{:#?}", f);
                tries += 1;
                calculate_and_sleep(&Thing::Preset, preset_id, &e, &tries).await;
            }
            Ok(caf) => {
                info!(
                    "got a site forum instance (aka prefix or caf) {} called {}",
                    preset_id, &caf.settings.title_welcome
                );
                return Some(caf);
            }
        }
    }
}

pub async fn get_forum_index_retry(
    forum_id: &String,
    page: Option<String>,
) -> Option<GetForumResult> {
    debug!("get_forum_index_retry({:?}, {:?})", forum_id, page);
    let mut tries = 1;
    loop {
        let maybe_gfr = SEE
            .get_forum(exposed_session!(), &forum_id, page.as_ref())
            .await;
        if maybe_gfr.is_err() {
            let e = maybe_gfr.unwrap_err();
            let f = format!("Subforum: {}, Try #{}: {}", forum_id, tries, e);
            warn!("{:#?}", &f);
            //The user doesn't have access to the forum. This isn't fatal for extracting what we can; log it and keep going
            if e.to_string().contains("noaccess")
                || e.to_string().contains("thread has been moved")
                || e.to_string().contains("The result is empty")
            {
                info!("Continuing anyway because this is not fatal. Your extraction may be incomplete.");
                return None;
            }
            if tries >= 5 {
                if state!().keep_going {
                    return None;
                } else {
                    panic!("{}", f);
                }
            }

            tries += 1;
            calculate_and_sleep(&Thing::ForumIndex, forum_id, &e, &tries).await;
        } else {
            return maybe_gfr.ok();
        }
    }
}

pub async fn get_thread_index_retry(
    thread_id: &String,
    page: Option<String>,
) -> Option<GetThreadResult> {
    debug!("get_thread_index_retry({}, {:?})", thread_id, page);
    let mut tries = 1;
    loop {
        let maybe_gtr = SEE
            .get_thread(exposed_session!(), thread_id, page.as_ref())
            .await;
        match maybe_gtr {
            Err(e) => {
                let f = format!("Thread: {}, Try #{}: {}", thread_id, tries, e);
                warn!(f);
                //The user doesn't have access to the forum. This isn't fatal for extracting what we can; log it and keep going
                if e.to_string().contains("noaccess")
                    || e.to_string().contains("thread has been moved")
                    || e.to_string().contains("The result is empty")
                {
                    warn!("Continuing anyway because this is not fatal. Your extraction may be incomplete.");
                    return None;
                }
                if tries >= 5 {
                    if state!().keep_going {
                        return None;
                    } else {
                        panic!("{}", f);
                    }
                }

                tries += 1;
                calculate_and_sleep(&Thing::Thread, thread_id, &e, &tries).await;
            }
            Ok(gtr) => {
                return Some(gtr);
            }
        }
    }
}

pub async fn get_subforums(subforum_ids: Vec<&String>) -> Vec<GetForumResult> {
    debug!("get_subforums({:#?})", subforum_ids);
    let mut retval: Vec<GetForumResult> = vec![];
    let mut page_map: HashMap<String, u32> = HashMap::new();
    let mut futs = FuturesUnordered::new();

    //Queue request for first page for every subforum index list.
    for forum_id in subforum_ids {
        futs.push(get_forum_index_retry(forum_id, None));
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
        info!("Got Page 1 of the Forum Index for {}", y.forum.forum_id);
        whoa(&mut arl).await;
        retval.push(y);
    }

    //Queue request for every remaining page for every subforum index list.
    for (forum_id, pages) in page_map.iter_mut() {
        let mut page = 2;
        while page <= *pages {
            futs.push(get_forum_index_retry(forum_id, Some(page.to_string())));
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
        info!(
            "Got Page {:?}/{:?} for Forum {} from Preset {}",
            xu.page, xu.pages, xu.forum.forum_id, xu.forum.preset_id
        );
        retval.push(xu);
        whoa(&mut arl).await;
    }

    return retval;
}

pub async fn get_threads(mut thread_ids: Vec<String>) -> Vec<GetThreadResult> {
    debug!("get_threads({:#?})", thread_ids);
    let mut retval = vec![];
    let mut page_map: HashMap<String, u32> = HashMap::new();
    let mut futs = FuturesUnordered::new();

    //Queue request for first page for every thread index list.
    for thread_id in thread_ids.iter_mut() {
        futs.push(get_thread_index_retry(thread_id, None));
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
        info!(
            "Got Page 1 of Thread {} from Forum {}",
            y.thread.thread_id, &y.thread.forum_id
        );
        retval.push(y);
    }

    //Queue request for every remaining page for every thread index list.
    for (thread_id, pages) in page_map.iter_mut() {
        let mut page = 2;
        while page <= *pages {
            futs.push(get_thread_index_retry(thread_id, Some(page.to_string())));
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
        info!(
            "Got Thread {} from Forum {}",
            xu.thread.thread_id, xu.thread.forum_id
        );
        retval.push(xu);
        whoa(&mut arl).await;
    }

    return retval;
}

pub async fn save_preset(preset_id: &String, caf: &GetCafResult) {
    debug!("save_preset({})", preset_id);
    let categories = &caf.category_names;

    for (cid, cn) in categories {
        let am = category_names::ActiveModel {
            category_id: Set(cid.to_string()),
            category_name: Set(cn.to_string()),
        };
        dumbsert!(
            category_names::Entity,
            &am,
            category_names::Column::CategoryId,
            "Error saving category to database",
            true
        );
    }

    let am = forum_presets::ActiveModel {
        preset_id: Set(preset_id.to_string()),
        title_welcome: Set(caf.settings.title_welcome.clone()),
        total_threads: Set(parse_number(&caf.total_threads)
            .unwrap()
            .try_into()
            .unwrap()),
        total_posts: Set(parse_number(&caf.total_posts).unwrap().try_into().unwrap()),
    };
    dumbsert!(
        forum_presets::Entity,
        &am,
        forum_presets::Column::PresetId,
        "Error saving preset to database",
        true
    );
}

pub async fn save_subforum(gfr: &GetForumResult) {
    debug!("save_subforum({})", gfr.forum.forum_id);

    let am = subforums::ActiveModel {
        title_welcome: Set(gfr.forum.title_welcome.clone()),
        preset_id: Set(gfr.forum.preset_id.clone()),
        category_id: Set(gfr.forum.category_id.clone()),
        category_name: Set(gfr.forum.category_name.clone()),
        forum_id: Set(gfr.forum.forum_id.clone()),
        forum_name: Set(gfr.forum.forum_name.clone()),
        forum_description: Set(gfr.forum.forum_description.clone()),
        parent_id: Set(gfr.forum.parent_id.clone()),
        forum_type: Set(gfr.forum.forum_type.clone()),
    };
    dumbsert!(
        subforums::Entity,
        &am,
        subforums::Column::ForumId,
        "Error saving subforum to database",
        true
    );

    let all_threads = gfr
        .threads
        .to_owned()
        .into_iter()
        .chain(gfr.announcement_global.to_owned().into_iter())
        .chain(gfr.announcement_local.to_owned().into_iter())
        .chain(gfr.sticky.to_owned().into_iter())
        .chain(gfr.notices.to_owned().into_iter());

    for thread in all_threads {
        let am = forum_threads::ActiveModel {
            thread_id: Set(thread.thread_id.clone()),
            thread_subject: Set(thread.thread_subject.clone()),
            thread_views: Set(thread.thread_views.clone()),
            thread_type: Set(thread.thread_type.clone()),
            thread_status: Set(thread.thread_status.clone()),
            forum_id: Set(gfr.forum.forum_id.clone()),
            username: Set(thread.username.clone()),
            category_id: Set(gfr.forum.category_id.clone()),
        };
        dumbsert!(
            forum_threads::Entity,
            &am,
            forum_threads::Column::ThreadId,
            "Error saving forum thread to database",
            true
        );
    }
}

pub async fn save_threads(gtrs: &Vec<GetThreadResult>) {
    debug!("save_threads()");
    for gtr in gtrs {
        for post in &gtr.posts {
            let am = forum_posts::ActiveModel {
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
            };
            dumbsert!(
                forum_posts::Entity,
                &am,
                forum_posts::Column::PostId,
                "Error saving post to database",
                true
            );
        }
    }
}

pub async fn get_forums() -> anyhow::Result<()> {
    let mut dones: HashSet<String> = HashSet::new();
    for caf_id in state!().forum_ids.as_ref().unwrap() {
        let subforum_results;
        let maybe_caf = get_preset_retry(caf_id).await;
        if maybe_caf.is_none() {
            continue;
        }

        let caf = maybe_caf.unwrap(); //Guaranteed to succeed
        save_preset(caf_id, &caf).await;
        let maybe_sfis = &state!().subforum_ids;
        let mut all_subforums: Vec<String> = vec![];

        //Add all the subforums.
        match caf.subforums {
            SubforumType::MapSubforum(m) => {
                all_subforums.extend(m.keys().cloned());
                for sfs in m.values() {
                    for sf in sfs {
                        all_subforums.push(sf.forum_id.clone());
                    }
                }
            }
            SubforumType::SeqSubforum(s) => {
                for x in s {
                    all_subforums.push(x.clone());
                }
            }
        }

        //Add the "categories" top-level forums.
        for foru in caf.categories.values() {
            for (fid, _) in foru {
                all_subforums.push(fid.clone());
            }
        }

        all_subforums.sort();
        all_subforums.dedup();

        //Call Forum.getForum for every CAF (only for allowed subforums).
        let allowed_subforums =
            Vec::from_iter(all_subforums.iter().filter(|subforum_id| match maybe_sfis {
                Some(sfis) => {
                    if sfis.len() == 0 {
                        debug!("subforum_ids.len() == 0");
                        return true;
                    } else {
                        debug!("subforum_ids contains check on {}", subforum_id);
                        return sfis.contains(subforum_id);
                    }
                }
                None => {
                    return true;
                }
            }));
        info!(
            "*** Total number of forums and subforums to scan: {}",
            allowed_subforums.len()
        );
        subforum_results = get_subforums(allowed_subforums).await;
        //Call Forum.getThread for every GFR.
        for gfr in &subforum_results {
            save_subforum(&gfr).await;
            let mut inval = vec![];
            for inv in &gfr.threads {
                inval.push(inv.thread_id.clone());
            }
            debug!(
                "Sticky thread count for GFR page {} for forum {}: {}",
                gfr.page,
                gfr.forum.forum_id,
                gfr.sticky.len()
            );
            for inv in &gfr.sticky {
                inval.push(inv.thread_id.clone());
            }
            for inv in &gfr.notices {
                inval.push(inv.thread_id.clone());
            }
            for inv in &gfr.announcement_local {
                inval.push(inv.thread_id.clone());
            }
            for inv in &gfr.announcement_global {
                if !dones.contains(&inv.thread_id) {
                    inval.push(inv.thread_id.clone());
                    dones.insert(inv.thread_id.clone());
                }
            }
            let threads = get_threads(inval).await;
            save_threads(&threads).await;

            if state!().do_images {
                for thread in threads {
                    for post in thread.posts {
                        get_images(post.post_id.clone(), post.post_content.clone()).await;
                    }
                }
            }
        }
    }
    info!("*** Done extracting forums.");
    Ok(())
}
