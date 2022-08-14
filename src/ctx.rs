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
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
//! App data: database connections, etc.
use std::sync::Arc;
use std::thread;

use argon2_creds::{Config, ConfigBuilder, PasswordPolicy};

//use crate::errors::ServiceResult;
use crate::settings::Settings;
/// App data
pub struct Ctx {
    //    /// database ops defined by db crates
    //    pub db: BoxDB,
    /// credential management configuration
    pub creds: Config,
    /// app settings
    pub settings: Settings,
    pub source_code: String,
}

impl Ctx {
    pub fn get_creds() -> Config {
        ConfigBuilder::default()
            .username_case_mapped(true)
            .profanity(true)
            .blacklist(true)
            .password_policy(PasswordPolicy::default())
            .build()
            .unwrap()
    }

    #[cfg(not(tarpaulin_include))]
    /// create new instance of app data
    pub async fn new(s: &Settings) -> ArcCtx {
        let creds = Self::get_creds();
        let c = creds.clone();

        #[allow(unused_variables)]
        let init = thread::spawn(move || {
            log::info!("Initializing credential manager");
            c.init();
            log::info!("Initialized credential manager");
        });

        //let db = match s.database.database_type {
        //    crate::settings::DBType::Maria => db::maria::get_data(Some(s.clone())).await,
        //    crate::settings::DBType::Postgres => db::pg::get_data(Some(s.clone())).await,
        //};

        #[cfg(not(debug_assertions))]
        init.join().unwrap();

        let source_code = {
            let mut url = s.source_code.clone();
            if !url.ends_with('/') {
                url.push('/');
            }
            let mut base = url::Url::parse(&url).unwrap();
            base = base.join("tree/").unwrap();
            base = base.join(crate::GIT_COMMIT_HASH).unwrap();
            base.into()
        };

        let data = Ctx {
            creds,
            //   db,
            settings: s.clone(),
            source_code,
        };

        Arc::new(data)
    }
}

pub type ArcCtx = Arc<Ctx>;
