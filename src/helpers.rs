/// Copyright (c) 2023, Sean McNamara <smcnam@gmail.com>.
/// All code in this repository is disjunctively licensed under [CC-BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/) and [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0).
/// Direct dependencies are believed to be under a license which allows downstream code to have these licenses.
use jsonrpsee::core::Error;
use std::time::Duration;
use tracing::warn;

#[derive(Debug)]
pub enum Thing {
    Preset,
    ForumIndex,
    Thread,
    Application,
    ApplicationList,
}

pub fn parse_number(val: &serde_json::Value) -> Option<u32> {
    match val {
        sea_orm::JsonValue::Null => None,
        sea_orm::JsonValue::Bool(_) => panic!("Expected number, got bool"),
        sea_orm::JsonValue::Number(n) => Some(n.as_u64().unwrap().try_into().unwrap()),
        sea_orm::JsonValue::String(s) => Some(s.parse::<u32>().unwrap()),
        sea_orm::JsonValue::Array(_) => panic!("Expected number, got array"),
        sea_orm::JsonValue::Object(_) => panic!("Expected number, got object"),
    }
}

//Slow down the calls ever so slightly to reduce the chance of being rate-limited
pub async fn whoa(arl: &mut u32) {
    tokio::time::sleep(Duration::from_millis((100 * *arl).into())).await;
    if *arl < 5 {
        *arl += 1;
    }
}

pub async fn calculate_and_sleep(thing: &Thing, thing_id: &String, e: &Error, tries: &u32) {
    let mut dur: u32 = 30;
    if e.to_string().contains("status code: 429") {
        dur = 30 + (60 * tries * tries); // 30 + 60x^2 quadratic backoff
        warn!("For {:?} {}: HTTP response code 429 means Enjin rate-limited us for going too fast! Waiting {} seconds.",
        thing, thing_id, dur);
    } else {
        warn!("For {:?} {}: Error {:?}", thing, thing_id, e);
    }
    tokio::time::sleep(Duration::from_secs(dur.into())).await;
}

#[macro_export]
macro_rules! dumbsert {
    ($ntt:ty, $model:expr, $column:expr, $error_msg:expr, $do_panic:expr) => {
        let dumbsert_rslt = <$ntt>::insert($model.to_owned())
            .on_conflict(
                // on conflict do nothing
                OnConflict::column($column).do_nothing().to_owned(),
            )
            .exec(&state!().conn)
            .await;
        if let Err(dr) = dumbsert_rslt {
            if let migration::DbErr::RecordNotInserted = dr {
                tracing::debug!("Skipping RecordNotInserted.");
                let dumbsert_update_rslt =
                    <$ntt>::update($model.to_owned()).exec(&state!().conn).await;
                if let Err(ddr) = dumbsert_update_rslt {
                    if let migration::DbErr::RecordNotUpdated = ddr {
                        tracing::debug!("Skipping RecordNotUpdated.");
                    } else {
                        if $do_panic {
                            panic!("UPDATE Code-up error: {}", ddr);
                        } else {
                            tracing::info!("Couldn't update record but continuing: {}", $error_msg);
                        }
                    }
                }
            } else {
                panic!("INSERT Code-up error: {}", dr);
            }
        };
    };
}
