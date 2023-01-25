/// Copyright (c) 2023, Sean McNamara <smcnam@gmail.com>.
/// All code in this repository is disjunctively licensed under [CC-BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/) and [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0).
/// Direct dependencies such as Rust, Diesel-rs, Hyper and jsonrpsee are licensed under the MIT or 3-clause BSD license, which allow downstream code to have any license.
use crate::schema::*;
use crate::structures::*;
use anyhow::bail;
use diesel::{ExpressionMethods, RunQueryDsl};
use jsonrpsee::core::Error;
use jsonrpsee::proc_macros::rpc;
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

pub async fn get_forums(state: &mut State) -> anyhow::Result<()> {
    let mut cafs: Vec<GetCafResult> = vec![];
    let mut threads: Vec<ForumThread> = vec![];
    let mut categories: &HashMap<String, String>;
    let conn = &mut state.conn;

    for caf_id in state.forum_ids.as_ref().unwrap() {
        let mut caf: GetCafResult;

        //XXX: Figure out how to write a closure that does this so I don't have to copy paste this 3 times
        let mut ctries = 0;
        'cafgeez: loop {
            let caff = state
                .client
                .get_categories_and_forums(state.session_id.as_ref().unwrap(), &caf_id)
                .await;
            let my_err: Error;
            match caff {
                Ok(c) => {
                    caf = c;
                    break 'cafgeez;
                }
                Err(e) => {
                    println!("{}", e);
                    my_err = e;
                }
            }

            if ctries >= 5 {
                bail!(my_err);
            }

            ctries += 1;

            tokio::time::sleep(Duration::from_secs(90)).await;
        }

        println!(
            "got a site forum instance (aka prefix or caf) {} called {}",
            caf_id, &caf.settings.title_welcome
        );
        categories = &caf.category_names;

        for (cid, cn) in categories {
            diesel::insert_or_ignore_into(category_names::table)
                .values((
                    category_names::category_id.eq(cid),
                    category_names::category_name.eq(cn),
                ))
                .execute(conn)
                .expect("Error saving new subforum");
        }

        for (_forum_id, subforums) in caf.subforums.iter_mut() {
            for subforum in subforums {
                match &state.subforum_ids {
                    Some(sfis) => {
                        if !sfis.contains(&subforum.forum_id) {
                            continue;
                        }
                    }
                    None => (),
                };
                let mut forum_curr_page: u32 = 1;
                let mut forum_pages: u32;

                //Loop through each page of the subforum thread index.
                'subforumpage: loop {
                    let mut sf: GetForumResult;

                    let mut gtries = 0;
                    'gfr: loop {
                        let sff = state
                            .client
                            .get_forum(
                                state.session_id.as_ref().unwrap(),
                                &subforum.forum_id,
                                Some(&forum_curr_page.to_string()),
                            )
                            .await;
                        let my_err: Error;
                        match sff {
                            Ok(c) => {
                                sf = c;
                                break 'gfr;
                            }
                            Err(e) => {
                                println!("{}", e);
                                my_err = e;
                            }
                        }

                        if gtries >= 5 {
                            if state.keep_going {
                                continue 'subforumpage;
                            } else {
                                bail!(my_err);
                            }
                        }

                        gtries += 1;
                        tokio::time::sleep(Duration::from_secs(90)).await;
                    }

                    println!(
                        "got page {}/{} of subforum {} called {}",
                        sf.page, sf.pages, sf.forum.forum_id, sf.forum.forum_name
                    );
                    if forum_curr_page == 1 {
                        diesel::insert_or_ignore_into(subforums::table)
                            .values(&sf.forum)
                            .execute(conn)
                            .expect("Error saving new subforum");
                    }
                    forum_pages = sf.pages;
                    'threadloop: for thread in sf.threads.iter_mut() {
                        let mut thread_curr_page: u32 = 1;
                        let mut thread_pages: u32;

                        //Loop through each post of a thread.
                        loop {
                            let mut gtr: GetThreadResult;

                            let mut ttries = 0;
                            'gtrgeez: loop {
                                let gtrr = state
                                    .client
                                    .get_thread(
                                        state.session_id.as_ref().unwrap(),
                                        &thread.thread_id,
                                        Some(&thread_curr_page.to_string()),
                                    )
                                    .await;
                                let my_err: Error;
                                match gtrr {
                                    Ok(c) => {
                                        gtr = c;
                                        break 'gtrgeez;
                                    }
                                    Err(e) => {
                                        println!("{}", e);
                                        my_err = e;
                                    }
                                }

                                if ttries >= 5 {
                                    if state.keep_going {
                                        continue 'threadloop;
                                    } else {
                                        bail!(my_err);
                                    }
                                }

                                ttries += 1;
                                tokio::time::sleep(Duration::from_secs(90)).await;
                            }

                            thread_pages = gtr.pages;
                            println!(
                                "got page {}/{} of a thread {} called {}",
                                thread_curr_page,
                                thread_pages,
                                gtr.thread.thread_id,
                                gtr.thread.thread_subject
                            );
                            for post in gtr.posts.iter_mut() {
                                println!(
                                    "got a post {} related to thread {}",
                                    post.post_id, thread.thread_id
                                );
                                post.thread_id = Some(thread.thread_id.clone());
                            }

                            diesel::insert_or_ignore_into(forum_posts::table)
                                .values(&gtr.posts)
                                .execute(conn)
                                .expect("Error saving new posts");

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
