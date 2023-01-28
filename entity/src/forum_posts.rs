//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.7

use sea_orm::entity::prelude::*;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Deserialize)]
#[sea_orm(table_name = "forum_posts")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub post_id: String,
    pub post_time: String,
    pub post_content: String,
    pub post_user_id: String,
    pub last_edit_time: String,
    pub post_unhidden: String,
    pub post_admin_hidden: String,
    pub post_locked: String,
    pub last_edit_user: String,
    pub post_username: String,
    pub thread_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub type ForumPost = Model;
