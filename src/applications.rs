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
trait ApplicationsApi {
    #[method(name="Applications.getList", param_kind=map)]
    async fn get_applications_list(
        &self,
        session_id: &String,
        _type: &String,
        page: Option<u32>,
        site_id: Option<u32>, //int|bool [optional]
        application_form_id: Option<u32>, //int|bool [optional]
        unread_only: Option<bool>,
        search: Option<String>,
        limit: Option<u32>,
    ) -> Result<GetApplicationsListResult, Error>;

    #[method(name="Applications.getTypes", param_kind=map)]
    async fn get_application_types(
        &self,
        session_id: &String,
    ) -> Result<GetApplicationTypesResult, Error>;

    #[method(name="Applications.getApplication", param_kind=map)]
    async fn get_application(
        &self,
        session_id: &String,
        application_id: u32
    ) -> Result<GetApplicationResult, Error>;

}