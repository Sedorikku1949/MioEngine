use serenity::{ model::prelude::Message, http::CacheHttp, prelude::Context };
use crate::{Storage, utils};
use super::{CommandData, CommandError};

pub (in crate::handle) async fn execute(
  ctx: &Context,
  http: &impl CacheHttp,
  message: &Message,
  storage: &Storage,
  _command: &CommandData
) -> Result<(), CommandError> {
  if let Some(act_shard) = storage.latency.get(&ctx.shard_id) {
    if act_shard.ping.as_nanos() > 0 {
      let msg = message.reply(
        http,
        format!("🏓 **Pong!**, j'ai une latence de `{l}ms` (shard: {id}) !", l = act_shard.ping.as_millis(), id = ctx.shard_id).as_str()
      ).await;
      if let Err(why) = msg {
        utils::error("MessageSender", "An error occured while sending the message", why.to_string().as_str());
      }
    } else {
      let _ = too_early(http, message).await;
      return Err(CommandError::TooEarly)
    }
  } else {
    let _ = too_early(http, message).await;
    return Err(CommandError::InvalidData)
  };

  Ok(())
}

async fn too_early(http: &impl CacheHttp, message: &Message) -> Result<(), String> {
  if let Err(why) = message.reply(http, "> 🦀 ** ** **Je démarre encore.**\nIl me faut encore 1m pour me réveiller complètement.").await {
    utils::error("MessageSender", "An error occured while sending the message", why.to_string().as_str());
    Err(why.to_string())
  } else { Ok(()) }
}
