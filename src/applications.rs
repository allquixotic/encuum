/// Copyright (c) 2023, Sean McNamara <smcnam@gmail.com>.
/// All code in this repository is disjunctively licensed under [CC-BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/) and [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0).
/// Direct dependencies are believed to be under a license which allows downstream code to have these licenses.
use crate::state;
use crate::exposed_session;
use crate::structures::*;
use entity::*;
use entity::applications::AppApp;
use jsonrpsee::proc_macros::rpc;
use sea_orm::{sea_query::OnConflict, EntityTrait, Set};
use secrecy::ExposeSecret;
use tracing::{info, debug, warn};
use std::{
    collections::{HashMap},
};

use crate::helpers::*;

#[rpc(client)]
trait ApplicationsApi {
    #[method(name="Applications.getList", param_kind=map)]
    async fn get_applications_list(
        &self,
        session_id: &String,
        r#type: &String,
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
    ) -> Result<Option<HashMap<String, String>>, Error>;

    #[method(name="Applications.getApplication", param_kind=map)]
    async fn get_application(
        &self,
        session_id: &String,
        application_id: u32
    ) -> Result<AppApp, Error>;

}

//Implement the function save_application to save an AppApp to the sqlite database using sea-orm.
pub async fn save_application(app: &AppApp) -> anyhow::Result<()> {
    applications::Entity::insert(applications::ActiveModel {
        application_id: Set(app.application_id.clone()),
        site_id: Set(app.site_id.clone()),
        preset_id: Set(app.preset_id.clone()),
        title: Set(app.title.clone()),
        user_ip: Set(app.user_ip.clone()),
        created: Set(app.created.clone()),
        username: Set(app.username.clone()),
        user_id: Set(app.user_id.clone()),
        user_data: Set(app.user_data.clone()),
    })
    .on_conflict(
        // on conflict do nothing
        OnConflict::column(applications::Column::ApplicationId)
            .do_nothing()
            .to_owned(),
    )
    .exec(&state!().conn)
    .await
    .expect("Code-up error: can't insert an application into the database!");
    Ok(())
}

pub async fn get_app_list(types: &HashMap<String, String>) -> anyhow::Result<Vec<String>> {
    let mut retval = vec![];
    for (k, _v) in types {
        //Enumerate each page of the application list and add the application ID of each item to retval.
        let mut total = 0;
        let mut page = 1;
        let mut retries: u32 = 0;
        loop {
            let maybe_gar_result = SEE.get_applications_list(exposed_session!(), k, Some(page), None, None, None, None, None).await;
            if let Ok(gar_result) = maybe_gar_result {
                if let Some(gars) = gar_result.items {
                    for gar in gars {
                        if let Some(appid) = gar.application_id {
                            retval.push(appid);
                        }
                    }
                }
                if gar_result.total.is_none() || gar_result.total.clone().unwrap().parse::<u32>().is_err() {
                    break;
                }
                let resp_tot = gar_result.total.unwrap().parse::<u32>().unwrap();
                total += resp_tot;
                if total >= resp_tot {
                    break;
                }
                page += 1;
            }
            else if let Err(e) = maybe_gar_result {
                retries += 1;
                calculate_and_sleep(&Thing::ApplicationList, &page.to_string(), &e, &retries).await;
                if retries > 5 {
                    break;
                }
            }
        }
    }
    Ok(retval)
}

//Fetch all of the applications of every type from the website.
pub async fn get_apps() -> anyhow::Result<()> {
    let types = SEE.get_application_types(exposed_session!()).await?.expect("No application types found - this is probably a bug");
    let gars = get_app_list(&types).await?;
    for gar in gars {
        let app = SEE.get_application(exposed_session!(), gar.parse::<u32>().unwrap()).await?;
        save_application(&app).await?;
        info!("Saved application {}", gar);
    }
    Ok(())
}