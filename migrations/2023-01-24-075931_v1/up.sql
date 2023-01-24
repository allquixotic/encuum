CREATE TABLE subforums (
    title_welcome TEXT,
    preset_id TEXT NOT NULL,
    category_id TEXT NOT NULL,
    category_name TEXT NOT NULL,
    forum_id TEXT PRIMARY KEY,
    forum_name TEXT NOT NULL,
    forum_description TEXT NOT NULL,
    parent_id TEXT NOT NULL,
    forum_type TEXT NOT NULL
);

CREATE TABLE forum_posts (
    post_id TEXT PRIMARY KEY,
    post_time TEXT NOT NULL,
    post_content TEXT NOT NULL,
    post_user_id TEXT NOT NULL,
    last_edit_time TEXT NOT NULL,
    post_unhidden TEXT NOT NULL,
    post_admin_hidden TEXT NOT NULL,
    post_locked TEXT NOT NULL,
    last_edit_user TEXT NOT NULL,
    post_username TEXT NOT NULL,
    thread_id TEXT
);

CREATE TABLE forum_threads (
    thread_id TEXT PRIMARY KEY,
    thread_subject TEXT NOT NULL,
    thread_views TEXT NOT NULL,
    thread_type TEXT NOT NULL,
    thread_status TEXT NOT NULL,
    forum_id TEXT NOT NULL,
    username TEXT,
    category_id TEXT NOT NULL
);

CREATE TABLE category_names (
    category_id TEXT PRIMARY KEY,
    category_name TEXT NOT NULL
);

CREATE TABLE forum_presets (
    preset_id TEXT PRIMARY KEY,
    title_welcome TEXT NOT NULL,
    total_threads INTEGER NOT NULL,
    total_posts INTEGER NOT NULL
);

CREATE TABLE images (
    image_url TEXT PRIMARY KEY,
    image_content BLOB
);