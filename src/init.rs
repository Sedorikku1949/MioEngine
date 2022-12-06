use std::{
  fs::OpenOptions,
  io::Read
};
use serde::Deserialize;

use crate::utils;

#[derive(Deserialize)]
pub struct Config {
  pub client: Client,
  pub params: Params,
  pub security: Security,
  pub i18n: I18n
}

#[derive(Deserialize)]
pub struct Client {
  pub version: String,
  pub build_type: String,
  pub dev: bool
}

#[derive(Deserialize)]
pub struct Params {
  pub status: Vec<Status>,
  pub auto_status: bool,
  pub prefix: String,
  pub status_time: i32
}

#[derive(Deserialize, Debug, Clone)]
pub struct Status {
  pub status_type: String,
  pub message: String
}

#[derive(Deserialize)]
pub struct Security {
  pub rewrite_archive_if_invalid: bool,
  pub auto_save_archive: bool
}

#[derive(Deserialize)]
pub struct I18n {
  pub locales_dir: String
}


fn parse_config_file(content: &str) -> Result<Config, toml::de::Error> {
  let parsed: Result<Config, toml::de::Error> = toml::from_str(content);
  parsed
}
 
pub fn read_config() -> Result<Config, ()> {
  let config = OpenOptions::new().read(true).write(true).create(true).open(RELATIVE_CONFIG_DIR);
  match config {
    Ok(mut cnf_file) => {
      let mut cnf_body: String = String::new();
      let _ = cnf_file.read_to_string(&mut cnf_body);
      match parse_config_file(&cnf_body.as_str()) {
        Ok(cnf) => {
          utils::success("ConfigReader", "Configuration successfully loaded");
          Ok(cnf)
        },
        Err(err) => {
          utils::error("ConfigReader", "cannot parse the configuration", err.to_string().as_str());
          Err(())
        }
      }
    }
    Err(err) => {
      utils::error("ConfigReader", "cannot read the config file", err.to_string().as_str());
      Err(())
    }
  }
}


pub const RELATIVE_CONFIG_DIR: &str = "./config.toml";