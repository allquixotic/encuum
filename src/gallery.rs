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
trait GalleryApi {
    #[method(name="Gallery.getAlbums", param_kind=map)]
    async fn get_gallery_albums(
        &self,
        session_id: &String,
    ) -> Result<GetGalleryAlbumsResult, Error>;

    #[method(name="Gallery.getAlbum", param_kind=map)]
    async fn get_gallery_album(
        &self,
        session_id: &String,
        preset_id: u32,
        album_id: u32,
    ) -> Result<GetGalleryAlbumResult, Error>;

}