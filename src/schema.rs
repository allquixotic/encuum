// @generated automatically by Diesel CLI.

diesel::table! {
    subforums (forum_id) {
        title_welcome -> Nullable<Text>,
        preset_id -> Text,
        category_id -> Text,
        category_name -> Text,
        forum_id -> Nullable<Text>,
        forum_name -> Text,
        forum_description -> Text,
        parent_id -> Text,
        forum_type -> Text,
    }
}
