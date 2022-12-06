use std::fmt::Display;

use serenity::{model::prelude::Message, http::CacheHttp, prelude::Context};
use crate::{ Storage, utils };

// ==================================
// handler

#[derive(Debug)]
#[allow(dead_code)]
pub (in crate::handle) struct CommandData {
  pub name: String,
  pub prefix: String,
  pub args: Vec<String>
}

impl CommandData {
  fn new(prefix: &String, content: &String) -> Result<CommandData, ()> {
    let splitted = content[(prefix.len())..].split(" ")
      .filter(|p| p.trim().len() > 0)
      .map(|w| w.trim().to_string())
      .collect::<Vec<String>>();

    if splitted.len() > 0 {
      Ok(CommandData { name: splitted[0].clone(), prefix: prefix.clone(), args: splitted[1..].to_vec() })
    } else {
      Err(())
    }
  }
}

fn after_execution(message: &Message, storage: &Storage, cmd: &CommandData){
  if storage.handler_state.is_dev() {
    utils::send(
      "CommandHandler",
      format!("Command \x1b[33m{n}\x1b[0m used by \x1b[35m{a}\x1b[0m", n = cmd.name, a = message.author.tag()).as_str(),
      36
    );
  }
}

pub async fn execute(
  ctx: &Context,
  http: &impl CacheHttp,
  message: &Message,
  storage: &Storage
) {
  if message.content.trim().len() < 1 || message.author.bot || !message.content.starts_with(&storage.client.prefix) { return; }

  match CommandData::new(&storage.client.prefix, &message.content) {
    Ok(cmd) => async {
      exec_command(ctx, http, message, storage, cmd).await;
    },
    // Cannot found any command after the prefix
    Err(_) => { return; }
  }.await;
}

async fn exec_command(
  ctx: &Context,
  http: &impl CacheHttp,
  message: &Message,
  storage: &Storage,
  command: CommandData
){
  let cmd_result: Result<Result<(), CommandError>, CommandError> = match &command.name.as_str() {
    &"ping" => {
      Ok(ping::execute(ctx, http, message, storage, &command).await)
    },
    _ => Err(CommandError::CommandNotFound)
  };

  match cmd_result {
    Ok(result) => {
      if let Err(err) = result {
        match err {
          CommandError::TooEarly | CommandError::CommandNotFound => {
            utils::warn_with_cause("CommandHandler", "An error occured while executing the command", err.as_str())
          }
          CommandError::TreatedException => {}
          _ => {
            utils::error("CommandHandler", "An error occured while executing the command", err.as_str())
          }
        }
      } else {
        after_execution(message, storage, &command);
      }
    },
    Err(err) => {
      utils::warn_with_cause("CommandHandler", "An error occured while executing the command", err.as_str())
    }
  }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum CommandError {
  MessageNotSent,
  InvalidData,
  NoPermissions,
  TreatedException,
  TooEarly,
  CommandNotFound,
  Unknown
}

impl CommandError {
  pub fn as_str(&self) -> &str {
    match self {
      CommandError::MessageNotSent => "MessageNotSent",
      CommandError::InvalidData => "InvalidData",
      CommandError::NoPermissions => "NoPermissions",
      CommandError::TreatedException => "TreatedException",
      CommandError::TooEarly => "TooEarly",
      CommandError::CommandNotFound => "CommandNotFound",
      _ => "Unknown"
    }
  }
}

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}


// ==================================
// declare commands
pub mod ping;