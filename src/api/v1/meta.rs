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

use actix_web::{web, HttpResponse, Responder};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};

use crate::AppCtx;
use crate::{GIT_COMMIT_HASH, VERSION};

#[derive(Clone, Debug, Deserialize, Builder, Serialize)]
pub struct BuildDetails {
    pub version: &'static str,
    pub git_commit_hash: &'static str,
    pub source_code: String,
}

pub mod routes {
    pub struct Meta {
        pub build_details: &'static str,
        pub health: &'static str,
    }

    impl Meta {
        pub const fn new() -> Self {
            Self {
                build_details: "/api/v1/meta/build",
                health: "/api/v1/meta/health",
            }
        }
    }
}

/// emits build details of the bninary
#[actix_web_codegen_const_routes::get(path = "crate::API_V1_ROUTES.meta.build_details")]
async fn build_details(ctx: AppCtx) -> impl Responder {
    let build = BuildDetails {
        version: VERSION,
        git_commit_hash: GIT_COMMIT_HASH,
        source_code: ctx.source_code.clone(),
    };
    HttpResponse::Ok().json(build)
}

#[derive(Clone, Debug, Deserialize, Builder, Serialize)]
/// Health check return datatype
pub struct Health {
    db: bool,
}

/// checks all components of the system
#[actix_web_codegen_const_routes::get(path = "crate::API_V1_ROUTES.meta.health")]
async fn health() -> impl Responder {
    //   let mut resp_builder = HealthBuilder::default();

    //   resp_builder.db(data.db.ping().await);

    HttpResponse::Ok() //.json(resp_builder.build().unwrap())
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(build_details);
    cfg.service(health);
}

#[cfg(test)]
pub mod tests {
    use actix_web::{http::StatusCode, test, App};

    use crate::api::v1::services;
    use crate::*;

    #[actix_rt::test]
    async fn build_details_works() {
        let settings = Settings::new().unwrap();
        let ctx = AppCtx::new(crate::ctx::Ctx::new(&settings).await);
        let app = test::init_service(App::new().app_data(ctx.clone()).configure(services)).await;

        let resp = test::call_service(
            &app,
            test::TestRequest::get()
                .uri(API_V1_ROUTES.meta.build_details)
                .to_request(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    //    #[actix_rt::test]
    //    async fn health_works_pg() {
    //        let data = crate::tests::pg::get_data().await;
    //        health_works(data).await;
    //    }
    //
    //    #[actix_rt::test]
    //    async fn health_works_maria() {
    //        let data = crate::tests::maria::get_data().await;
    //        health_works(data).await;
    //    }
    //
    //    pub async fn health_works(data: ArcCtx) {
    //        println!("{}", API_V1_ROUTES.meta.health);
    //        let data = &data;
    //        let app = get_app!(data).await;
    //
    //        let resp = test::call_service(
    //            &app,
    //            test::TestRequest::get()
    //                .uri(API_V1_ROUTES.meta.health)
    //                .to_request(),
    //        )
    //        .await;
    //        assert_eq!(resp.status(), StatusCode::OK);
    //
    //        let health_resp: Health = test::read_body_json(resp).await;
    //        assert!(health_resp.db);
    //        assert_eq!(health_resp.redis, Some(true));
    //    }
}
