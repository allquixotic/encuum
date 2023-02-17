use crate::dumbsert;
use crate::exposed_session;
/// Copyright (c) 2023, Sean McNamara <smcnam@gmail.com>.
/// All code in this repository is disjunctively licensed under [CC-BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/) and [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0).
/// Direct dependencies are believed to be under a license which allows downstream code to have these licenses.
use crate::state;
use crate::structures::*;
use entity::applications::AppApp;
use entity::*;
use jsonrpsee::proc_macros::rpc;
use sea_orm::{sea_query::OnConflict, EntityTrait, Set};
use secrecy::ExposeSecret;
use std::collections::HashMap;
use tracing::{info, warn};

use crate::helpers::*;

#[rpc(client)]
trait ApplicationsApi {
    #[method(name="Applications.getList", param_kind=map)]
    async fn get_applications_list(
        &self,
        session_id: &String,
        r#type: &String,
        page: Option<u32>,
        site_id: Option<u32>,             //int|bool [optional]
        application_form_id: Option<u32>, //int|bool [optional]
        unread_only: Option<bool>,
        search: Option<String>,
        limit: Option<u32>,
    ) -> Result<GetApplicationsListResult, Error>;

    #[method(name="Applications.getTypes", param_kind=map)]
    async fn get_application_types(
        &self,
        session_id: &String,
    ) -> Result<Option<HashMap<String, String>>, Error>;

    #[method(name="Applications.getApplication", param_kind=map)]
    async fn get_application(
        &self,
        session_id: &String,
        application_id: u32,
    ) -> Result<AppApp, Error>;
}

//Implement the function save_application to save an AppApp to the sqlite database using sea-orm.
pub async fn save_application(app: &AppApp) -> anyhow::Result<()> {
    let modd = applications::ActiveModel {
        application_id: Set(app.application_id.clone()),
        site_id: Set(app.site_id.clone()),
        preset_id: Set(app.preset_id.clone()),
        title: Set(app.title.clone()),
        user_ip: Set(app.user_ip.clone()),
        created: Set(app.created.clone()),
        username: Set(app.username.clone()),
        user_id: Set(app.user_id.clone()),
        user_data: Set(app.user_data.clone()),
    };
    dumbsert!(
        applications::Entity,
        &modd,
        applications::Column::ApplicationId,
        "Error saving application to database",
        true
    );
    Ok(())
}

pub async fn get_app_list(types: &HashMap<String, String>) -> anyhow::Result<Vec<String>> {
    let mut retval = vec![];
    for (k, _v) in types {
        //Enumerate each page of the application list and add the application ID of each item to retval.
        let mut apps = vec![];
        let mut claimed_total: u32 = 0;
        let mut page = 1;
        let mut retries: u32 = 0;
        loop {
            let maybe_gar_result = SEE
                .get_applications_list(
                    exposed_session!(),
                    k,
                    Some(page),
                    None,
                    None,
                    None,
                    None,
                    None,
                )
                .await;
            if let Ok(gar_result) = maybe_gar_result {
                if let Some(gars) = gar_result.items {
                    for gar in gars {
                        if let Some(appid) = gar.application_id {
                            apps.push(appid);
                        }
                    }
                }
                if gar_result.total.is_none()
                    || gar_result.total.clone().unwrap().parse::<u32>().is_err()
                {
                    warn!("Total applications is not a number! This is probably a bug.");
                    break;
                }
                //I'll hold you to the total you give me on the first call.
                if page == 1 {
                    let resp_tot = gar_result.total.unwrap().parse::<u32>().unwrap();
                    claimed_total = resp_tot;
                }
                info!(
                    "So far, got {} applications of type {}; Enjin promised us {}",
                    apps.len(),
                    k,
                    claimed_total
                );
                if apps.len() >= claimed_total.try_into().unwrap() {
                    info!(
                        "END OF APP CATEGORY: Got {} applications of type {}; Enjin promised us {}",
                        apps.len(),
                        k,
                        claimed_total
                    );
                    break;
                }
                page += 1;
            } else if let Err(e) = maybe_gar_result {
                retries += 1;
                calculate_and_sleep(&Thing::ApplicationList, &page.to_string(), &e, &retries).await;
                if retries > 5 {
                    break;
                }
            }
        }
        retval.extend(apps);
    }
    Ok(retval)
}

//Fetch all of the applications of every type from the website.
pub async fn get_apps() -> anyhow::Result<()> {
    let types = SEE
        .get_application_types(exposed_session!())
        .await?
        .expect("No application types found - this is probably a bug");
    let gars = get_app_list(&types).await?;
    for gar in gars {
        let app = SEE
            .get_application(exposed_session!(), gar.parse::<u32>().unwrap())
            .await?;
        save_application(&app).await?;
        info!("Saved application {}", gar);
    }
    Ok(())
}
