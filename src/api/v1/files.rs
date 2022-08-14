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
use actix_multipart::Multipart;
use actix_web::HttpMessage;
use actix_web::{web, Error, HttpRequest, HttpResponse, Responder};
use actix_web_httpauth::middleware::HttpAuthentication;
use futures_util::TryStreamExt as _;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use super::httpauth;
use super::SignedInUser;
use super::API_V1_ROUTES;
use crate::AppCtx;

pub mod routes {
    use super::*;
    #[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
    pub struct Files {
        pub delete_dir: &'static str,
        pub upload_file: &'static str,
        pub index: &'static str,
    }
    impl Files {
        pub const fn new() -> Self {
            Self {
                delete_dir: "/api/v1/files/delete",
                upload_file: "/api/v1/files/upload",
                index: "/api/v1/files/",
            }
        }
    }
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(delete_dir);
    cfg.service(upload_file);
    cfg.service(index);
}

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
struct Dir {
    path: String,
}

#[actix_web_codegen_const_routes::delete(
    path = "API_V1_ROUTES.files.delete_dir",
    wrap = "HttpAuthentication::basic(httpauth)"
)]
async fn delete_dir(
    req: HttpRequest,
    ctx: AppCtx,
    payload: web::Json<Dir>,
) -> Result<impl Responder, Error> {
    let path = {
        let ext = req.extensions();
        let user = ext.get::<SignedInUser>().unwrap().clone();
        ctx.settings.files.get_path(&user.0, &payload.path)
    };

    if path.exists() {
        if path.is_dir() {
            fs::remove_dir_all(path).await?;
            Ok(HttpResponse::Ok().into())
        } else {
            Ok(HttpResponse::BadRequest().body("Path is not dir".to_string()))
        }
    } else {
        Ok(HttpResponse::NotFound().body("dir not found".to_string()))
    }
}

#[actix_web_codegen_const_routes::post(
    path = "API_V1_ROUTES.files.upload_file",
    wrap = "HttpAuthentication::basic(httpauth)"
)]
async fn upload_file(
    ctx: AppCtx,
    mut payload: Multipart,
    req: HttpRequest,
    query: web::Query<Dir>,
) -> Result<HttpResponse, Error> {
    let path = {
        let ext = req.extensions();
        let user = ext.get::<SignedInUser>().unwrap().clone();
        ctx.settings.files.get_path(&user.0, &query.path)
    };
    if !path.exists() {
        fs::create_dir_all(&path).await?;
    }

    // iterate over multipart stream
    while let Some(mut field) = payload.try_next().await? {
        // A multipart/form-data stream has to contain `content_disposition`
        let content_disposition = field.content_disposition();

        let filename = content_disposition.get_filename();

        if filename.is_none() {
            return Ok(HttpResponse::BadRequest().body("Filename is not present".to_string()));
        }
        let filename = filename.unwrap();
        let filepath = path.join(filename);

        let mut f = fs::File::create(filepath).await?;

        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.try_next().await? {
            f.write_all(&chunk).await?
        }
    }

    Ok(HttpResponse::Ok().into())
}
#[actix_web_codegen_const_routes::get(
    path = "API_V1_ROUTES.files.index",
    wrap = "HttpAuthentication::basic(httpauth)"
)]
async fn index() -> HttpResponse {
    let html = r#"<html>
        <head><title>Upload Test</title></head>
        <body>
            <form target="/" method="post" enctype="multipart/form-data">
                <input type="file" multiple name="file"/>
                <button type="submit">Submit</button>
            </form>
        </body>
    </html>"#;

    HttpResponse::Ok().body(html)
}

#[cfg(test)]
pub mod tests {
    use actix_web::{
        http::{header, StatusCode},
        test, App,
    };

    use super::*;
    use crate::*;

    #[actix_rt::test]
    async fn index_works() {
        //        const USERNAME: &str = "index_works";
        //        const PASSWORD: &str = "23k4j;123k4j1;l23kj4";
        let settings = Settings::new().unwrap();
        let creds = settings.files.creds.get(0).unwrap().clone();
        let auth = format!(
            "Basic {}",
            base64::encode(format!("{}:{}", creds.username, creds.password))
        );

        //        let settings = Settings::new().unwrap();
        let ctx = AppCtx::new(crate::ctx::Ctx::new(&settings).await);
        let app = test::init_service(
            App::new()
                .app_data(ctx.clone())
                .configure(crate::routes::services),
        )
        .await;

        let index_resp = test::call_service(
            &app,
            test::TestRequest::get()
                .append_header((header::AUTHORIZATION, auth))
                .uri(API_V1_ROUTES.files.index)
                .to_request(),
        )
        .await;
        assert_eq!(index_resp.status(), StatusCode::OK);
    }

    #[actix_rt::test]
    async fn delete_dir_works() {
        //        const USERNAME: &str = "index_works";
        //        const PASSWORD: &str = "23k4j;123k4j1;l23kj4";
        let settings = Settings::new().unwrap();
        let creds = settings.files.creds.get(0).unwrap().clone();
        let auth = format!(
            "Basic {}",
            base64::encode(format!("{}:{}", creds.username.clone(), creds.password))
        );

        const TEST_DIR_NAME: &str = "test-delete_dir_works";
        const TEST_FILE_NAME: &str = "test-delete_dir_works--file";
        const TEST_NON_EXIST_DIR: &str = "test-delete_dir_works--no-exist";

        let test_dir = settings.files.get_path(&creds.username, TEST_DIR_NAME);
        if !test_dir.exists() {
            tokio::fs::create_dir_all(&test_dir).await.unwrap();
        }

        let test_file = settings.files.get_path(&creds.username, TEST_FILE_NAME);
        if !test_file.exists() {
            let mut f = tokio::fs::File::create(test_file).await.unwrap();
            f.write_all(b"foo").await.unwrap();
        }

        let ctx = AppCtx::new(crate::ctx::Ctx::new(&settings).await);
        let app = test::init_service(
            App::new()
                .app_data(ctx.clone())
                .configure(crate::routes::services),
        )
        .await;

        let mut payload = Dir {
            path: TEST_FILE_NAME.into(),
        };

        let delete_dir_resp = test::call_service(
            &app,
            test::TestRequest::delete()
                .append_header((header::AUTHORIZATION, auth.clone()))
                .set_json(&payload)
                .uri(API_V1_ROUTES.files.delete_dir)
                .to_request(),
        )
        .await;
        assert_eq!(delete_dir_resp.status(), StatusCode::BAD_REQUEST);

        payload.path = TEST_NON_EXIST_DIR.into();
        let delete_dir_resp = test::call_service(
            &app,
            test::TestRequest::delete()
                .append_header((header::AUTHORIZATION, auth.clone()))
                .set_json(&payload)
                .uri(API_V1_ROUTES.files.delete_dir)
                .to_request(),
        )
        .await;
        assert_eq!(delete_dir_resp.status(), StatusCode::NOT_FOUND);

        payload.path = TEST_DIR_NAME.into();
        let delete_dir_resp = test::call_service(
            &app,
            test::TestRequest::delete()
                .append_header((header::AUTHORIZATION, auth))
                .set_json(&payload)
                .uri(API_V1_ROUTES.files.delete_dir)
                .to_request(),
        )
        .await;
        assert_eq!(delete_dir_resp.status(), StatusCode::OK);

        assert!(!test_dir.exists());
    }
}
