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
use std::path::Path;
use std::{env, path::PathBuf};

use config::{Config, ConfigError, Environment, File};
use log::warn;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    pub port: u32,
    pub domain: String,
    pub cookie_secret: String,
    pub ip: String,
    pub url_prefix: Option<String>,
    pub proxy_has_tls: bool,
}

impl Server {
    #[cfg(not(tarpaulin_include))]
    pub fn get_ip(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

//#[derive(Deserialize, Serialize, Display, PartialEq, Clone, Debug)]
//#[serde(rename_all = "lowercase")]
//pub enum DBType {
//    #[display(fmt = "postgres")]
//    Postgres,
//    #[display(fmt = "maria")]
//    Maria,
//}
//
//impl DBType {
//    fn from_url(url: &Url) -> Result<Self, ConfigError> {
//        match url.scheme() {
//            "mysql" => Ok(Self::Maria),
//            "postgres" => Ok(Self::Postgres),
//            _ => Err(ConfigError::Message("Unknown database type".into())),
//        }
//    }
//}
//
//#[derive(Debug, Clone, Deserialize)]
//pub struct Database {
//    pub url: String,
//    pub pool: u32,
//    pub database_type: DBType,
//}

#[derive(Debug, Clone, Deserialize)]
pub struct Creds {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Files {
    pub path: String,
    pub creds: Vec<Creds>,
}

impl Files {
    pub fn authenticate(&self, username: &str, password: &str) -> bool {
        self.creds
            .iter()
            .any(|c| c.username == username && c.password == password)
    }

    pub fn get_path(&self, username: &str, path: &str) -> PathBuf {
        Path::new(&self.path).join(username).join(path)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub debug: bool,
    pub commercial: bool,
    //    pub database: Database,
    pub server: Server,
    pub source_code: String,
    pub files: Files,
    pub allow_registration: bool,
}

#[cfg(not(tarpaulin_include))]
impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        const CURRENT_DIR: &str = "./config/default.toml";
        const ETC: &str = "/etc/dumbserve/config.toml";

        if let Ok(path) = env::var("DUMBSERVE_CONFIG") {
            s.merge(File::with_name(&path))?;
        } else if Path::new(CURRENT_DIR).exists() {
            // merging default config from file
            s.merge(File::with_name(CURRENT_DIR))?;
        } else if Path::new(ETC).exists() {
            s.merge(File::with_name(ETC))?;
        } else {
            log::warn!("configuration file not found");
        }

        s.merge(Environment::with_prefix("DUMBSERVE").separator("_"))?;

        check_url(&s);

        match env::var("PORT") {
            Ok(val) => {
                s.set("server.port", val).unwrap();
            }
            Err(e) => warn!("couldn't interpret PORT: {}", e),
        }

        //        match env::var("DATABASE_URL") {
        //            Ok(val) => {
        //                let url = Url::parse(&val).expect("couldn't parse Database URL");
        //                s.set("database.url", url.to_string()).unwrap();
        //                let database_type = DBType::from_url(&url).unwrap();
        //                s.set("database.database_type", database_type.to_string())
        //                    .unwrap();
        //            }
        //            Err(e) => {
        //                set_database_url(&mut s);
        //            }
        //        }

        // setting default values
        //    #[cfg(test)]
        //    s.set("database.pool", 2.to_string())
        //        .expect("Couldn't set database pool count");

        match s.try_into::<Self>() {
            Ok(val) => {
                std::fs::create_dir_all(&val.files.path).unwrap();
                Ok(val)
            },
            Err(e) => Err(ConfigError::Message(format!("\n\nError: {}. If it says missing fields, then please refer to https://github.com/mCaptcha/mcaptcha#configuration to learn more about how mcaptcha reads configuration\n\n", e))),
        }
    }
}

#[cfg(not(tarpaulin_include))]
fn check_url(s: &Config) {
    let url = s
        .get::<String>("source_code")
        .expect("Couldn't access source_code");

    Url::parse(&url).expect("Please enter a URL for source_code in settings");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creds_works() {
        let settings = Settings::new().unwrap();
        let mut creds = settings.files.creds.get(0).unwrap().clone();

        assert!(settings
            .files
            .authenticate(&creds.username, &creds.password));

        creds.username = "noexist".into();
        assert!(!settings
            .files
            .authenticate(&creds.username, &creds.password));

        let mut creds = settings.files.creds.get(0).unwrap().clone();

        creds.password = "noexist".into();
        assert!(!settings
            .files
            .authenticate(&creds.username, &creds.password));
    }
}

//#[cfg(not(tarpaulin_include))]
//fn set_database_url(s: &mut Config) {
//    s.set(
//        "database.url",
//        format!(
//            r"postgres://{}:{}@{}:{}/{}",
//            s.get::<String>("database.username")
//                .expect("Couldn't access database username"),
//            s.get::<String>("database.password")
//                .expect("Couldn't access database password"),
//            s.get::<String>("database.hostname")
//                .expect("Couldn't access database hostname"),
//            s.get::<String>("database.port")
//                .expect("Couldn't access database port"),
//            s.get::<String>("database.name")
//                .expect("Couldn't access database name")
//        ),
//    )
//    .expect("Couldn't set database url");
//}
