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
use actix_web::dev::ServiceRequest;
use actix_web::web;
use actix_web::Error;
use actix_web::HttpMessage;
use actix_web_httpauth::extractors::basic::BasicAuth;

pub mod files;
pub mod meta;

use crate::errors::*;
use crate::AppCtx;
use crate::SETTINGS;

pub const API_V1_ROUTES: routes::Routes = routes::Routes::new();

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SignedInUser(String);

pub async fn httpauth(
    req: ServiceRequest,
    credentials: BasicAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let _ctx: &AppCtx = req.app_data().unwrap();
    let username = credentials.user_id();
    let password = credentials.password().unwrap();
    if SETTINGS.files.authenticate(username, password) {
        {
            let mut ext = req.extensions_mut();
            ext.insert(SignedInUser(username.to_string()));
        }
        Ok(req)
    } else {
        let e = Error::from(ServiceError::Unauthorized);
        Err((e, req))
    }
}

pub fn services(cfg: &mut web::ServiceConfig) {
    files::services(cfg);
    meta::services(cfg);
}

pub mod routes {
    use crate::api::v1::files::routes::Files;
    use crate::api::v1::meta::routes::Meta;

    pub struct Routes {
        pub files: Files,
        pub meta: Meta,
    }

    impl Routes {
        pub const fn new() -> Self {
            Self {
                files: Files::new(),
                meta: Meta::new(),
            }
        }
    }
}
