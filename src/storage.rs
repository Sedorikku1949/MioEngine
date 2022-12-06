#![allow(dead_code)]

use std::{sync::Arc, collections::HashMap, time::Duration};
use chrono::{Utc, DateTime};
use serenity::prelude::{TypeMapKey, RwLock};
use crate::init::Config;

#[derive(Clone, Copy)]
pub enum ClientActivityType {
  Playing = 0,
  Streaming = 1,
  Listening = 2,
  Watching = 3,
  Unknown = !0,
}

impl ClientActivityType {
  pub fn is_unknown(&self) -> bool {
    match self {
      ClientActivityType::Unknown => true,
      _ => false
    }
  }
}

#[derive(Clone)]
pub struct Status {
  pub message: String,
  pub status_type: ClientActivityType
}

pub struct StatusManager {
  pub list: Vec<Status>,
  pub continue_status: bool,
  pub dev_status: Status,
  pub maintenance_status: Status,
  pub debug_mode_status: Status,
  pub streaming_url: String,
  pub status_time: u64
}

pub struct ClientData {
  pub prefix: String
}

pub enum HandlerStatus {
    InDev,
    DebugMode,
    ProdMode
}

impl HandlerStatus {
    pub fn as_i32(&self) -> i32 {
        match self {
            HandlerStatus::InDev => 0,
            HandlerStatus::DebugMode => 1,
            HandlerStatus::ProdMode => 2
        }
    }

    pub fn is_dev(&self) -> bool {
      match self {
        HandlerStatus::InDev | HandlerStatus::DebugMode => true,
        _ => false
      }
    }
}


#[derive(Debug, Clone)]
pub struct Latency {
  pub ping: Duration,
  pub warned: bool
}

pub struct Storage {
  pub maintenance: bool,
  pub dev: bool,
  pub debug: bool,
  pub status: StatusManager,
  pub client: ClientData,
  pub handler_state: HandlerStatus,
  pub latency: HashMap<u64, Latency>,
  pub process_start: DateTime<Utc>
}

impl TypeMapKey for Storage {
  type Value = Arc<RwLock<Storage>>;
}

impl Storage {
  pub fn new(config: &Config) -> Storage {
    Storage {
      maintenance: false,
      dev: config.client.dev,
      debug: false,
      client: ClientData { prefix: config.params.prefix.clone() },
      status: StatusManager {
        list: vec![
          Status { message: "a".to_string(), status_type: ClientActivityType::Listening },
          Status { message: "b".to_string(), status_type: ClientActivityType::Listening }
        ],
        continue_status: true,
        dev_status: Status { message: "⚙️ Mode développeur".to_string(), status_type: ClientActivityType::Watching },
        maintenance_status: Status { message: "🚧 Mode maintenance".to_string(), status_type: ClientActivityType::Watching },
        debug_mode_status: Status { message: "🔧 Mode debug".to_string(), status_type: ClientActivityType::Watching },
        streaming_url: "https://www.twitch.tv/sedorriku_".to_string(),
        status_time: config.params.status_time.clone() as u64
      },
      handler_state: if config.client.dev { HandlerStatus::InDev } else if false { HandlerStatus::DebugMode } else { HandlerStatus::ProdMode },
      latency: HashMap::new(),
      process_start: Utc::now()
    }
  }
}