/// Copyright (c) 2023, Sean McNamara <smcnam@gmail.com>.
/// All code in this repository is disjunctively licensed under [CC-BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/) and [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0).
/// Direct dependencies such as Rust, Diesel-rs, Hyper and jsonrpsee are licensed under the MIT or 3-clause BSD license, which allow downstream code to have any license.
use diesel::table;

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
