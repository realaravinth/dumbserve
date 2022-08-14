/*
 * Copyright (C) 2022  Aravinth Manivannan <realaravinth@batsense.net>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use std::env;

use actix_files::Files;
use actix_web::http::StatusCode;
use actix_web::web::JsonConfig;
use actix_web::{error::InternalError, middleware, App, HttpServer};
use log::info;

use lazy_static::lazy_static;

mod api;
mod ctx;
//mod db;
//mod docs;
#[cfg(not(tarpaulin_include))]
mod errors;
//#[macro_use]
//mod pages;
//#[macro_use]
mod routes;
mod settings;
//mod static_assets;
//#[cfg(test)]
//#[macro_use]
//mod tests;
//
pub use crate::ctx::Ctx;
//pub use crate::static_assets::static_files::assets::*;
pub use api::v1::API_V1_ROUTES;
//pub use docs::DOCS;
//pub use pages::routes::ROUTES as PAGES;
pub use settings::Settings;
//use static_assets::FileMap;

lazy_static! {
    pub static ref SETTINGS: Settings= Settings::new().unwrap();
//    pub static ref S: String = env::var("S").unwrap();
//    pub static ref FILES: FileMap = FileMap::new();
//    pub static ref JS: &'static str =
//        FILES.get("./static/cache/bundle/bundle.js").unwrap();
//    pub static ref CSS: &'static str =
//        FILES.get("./static/cache/bundle/css/main.css").unwrap();
//    pub static ref MOBILE_CSS: &'static str =
//        FILES.get("./static/cache/bundle/css/mobile.css").unwrap();
}

pub const COMPILED_DATE: &str = env!("COMPILED_DATE");
pub const GIT_COMMIT_HASH: &str = env!("GIT_HASH");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const PKG_HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

pub const CACHE_AGE: u32 = 604800;

use ctx::ArcCtx;
pub type AppCtx = actix_web::web::Data<ArcCtx>;

#[actix_web::main]
#[cfg(not(tarpaulin_include))]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "info");

    pretty_env_logger::init();

    info!(
        "{}: {}.\nFor more information, see: {}\nBuild info:\nVersion: {} commit: {}",
        PKG_NAME, PKG_DESCRIPTION, PKG_HOMEPAGE, VERSION, GIT_COMMIT_HASH
    );

    let settings = Settings::new().unwrap();
    let ctx = Ctx::new(&settings).await;
    let ctx = actix_web::web::Data::new(ctx);

    let ip = settings.server.get_ip();
    println!("Starting server on: http://{ip}");

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(
                middleware::DefaultHeaders::new().add(("Permissions-Policy", "interest-cohort=()")),
            )
            .wrap(middleware::Compress::default())
            .app_data(ctx.clone())
            .wrap(middleware::NormalizePath::new(
                middleware::TrailingSlash::Trim,
            ))
            .app_data(get_json_err())
            .configure(routes::services)
            .service(Files::new("/", "./tmp").show_files_listing())
    })
    .bind(ip)?
    .run()
    .await
}

#[cfg(not(tarpaulin_include))]
pub fn get_json_err() -> JsonConfig {
    JsonConfig::default().error_handler(|err, _| {
        //debug!("JSON deserialization error: {:?}", &err);
        InternalError::new(err, StatusCode::BAD_REQUEST).into()
    })
}
