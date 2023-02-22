/// Copyright (c) 2023, Sean McNamara <smcnam@gmail.com>.
/// All code in this repository is disjunctively licensed under [CC-BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/) and [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0).
/// Direct dependencies are believed to be under a license which allows downstream code to have these licenses.
use crate::structures::*;
use crate::helpers::*;
use entity::*;
use futures::{stream::FuturesUnordered, StreamExt};
use jsonrpsee::proc_macros::rpc;
use lazy_static::lazy_static;
use regex::Regex;
use sea_orm::{sea_query::OnConflict, EntityTrait, Set};
use secrecy::ExposeSecret;
use tracing::{info, debug, warn};
use std::{
    collections::{HashMap, HashSet},
};

#[rpc(client)]
trait WikiApi {
    #[method(name="Wiki.getCategories", param_kind=map)]
    async fn get_wiki_categories(
        &self,
        session_id: &String,
        preset_id: &String,
        hidden: Option<bool>
    ) -> Result<GetWikiCategoriesResult, Error>;

    #[method(name="Wiki.getFiles", param_kind=map)]
    async fn get_wiki_files(
        &self,
        session_id: &String,
        preset_id: &String,
    ) -> Result<GetWikiFilesResult, Error>;

    #[method(name="Wiki.getNoCategoryPages", param_kind=map)]
    async fn get_wiki_no_category_pages(
        &self,
        session_id: &String,
        preset_id: &String,
        limit: Option<u32>
    ) -> Result<GetWikiNoCategoryPagesResult, Error>;

    #[method(name="Wiki.getPageCommentData", param_kind=map)]
    async fn get_wiki_page_comment_data(
        &self,
        session_id: &String,
        preset_id: &String,
        title: &String
    ) -> Result<GetWikiPageCommentDataResult, Error>;

    #[method(name="Wiki.getPageHistory", param_kind=map)]
    async fn get_wiki_page_history(
        &self,
        session_id: &String,
        preset_id: &String,
        title: &String,
        limit: Option<u32>,
        from: Option<u64>,
        to: Option<u64>,
    ) -> Result<GetWikiPageHistoryResult, Error>;

    #[method(name="Wiki.getPageList", param_kind=map)]
    async fn get_wiki_page_list(
        &self,
        session_id: &String,
        preset_id: &String,
    ) -> Result<GetWikiPageListResult, Error>;

    #[method(name="Wiki.getPageTitle", param_kind=map)]
    async fn get_wiki_page_title(
        &self,
        session_id: &String,
        preset_id: &String,
        title: &String,
        prop: Option<Vec<String>>,
        oldid: Option<u64>,
        diff: Option<u64>,
    ) -> Result<GetWikiPageTitleResult, Error>;

}