/*  DECLARE MODULES */
pub mod utils;
mod init;
mod constants;
mod handle;
mod storage;
mod libs;
mod assets;


extern crate serde_derive;
extern crate rust_i18n;

/* import */
use std::{
    sync::Arc,
    time::Duration,
    collections::HashMap, process::exit
};
use chrono::Utc;
use libs::i18n;
//use mysql_async::prelude::Queryable;
use serenity::{
    async_trait,
    model::{ channel::Message, gateway::Ready, prelude::Activity  },
    prelude::*,
    client::bridge::gateway::ShardManager
};
use storage::{ Latency, Status, ClientActivityType };
use tokio::{
    signal,
    time::sleep
};

use crate::{
    constants::{check_comp_id, ARCHIVE_DIR},
    init::Config,
    storage::Storage, libs::security::archive::{ self, Archive }
};


struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<RwLock<ShardManager>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message){
        if msg.author.bot || msg.content.len() < 1 { return; };
        
        let storage_lock = {
            let data = ctx.data.read().await;
            data.get::<Storage>().expect("Expected Storage in TypeMap.").clone()
        };
        let storage = storage_lock.read().await;

        handle::commands::execute(&ctx, &ctx.http, &msg, &storage).await;

        drop(storage);
        drop(storage_lock);
    }

    async fn ready(&self, ctx: Context, ready: Ready){
        utils::success("Ready", format!("{} is ready", ready.user.name).as_str());

        let storage_lock = {
            let data = ctx.data.read().await;
            data.get::<Storage>().expect("Expected Storage in TypeMap.").clone()
        };
        let storage = storage_lock.read().await;

        let now = Utc::now();
        let start_time = now.timestamp_millis() - storage.process_start.timestamp_millis();
        utils::info(
            "MioEngine",
            format!("Process started in {}ms", start_time).as_str()
        );
    }
}

fn get_config() -> Config {
    match init::read_config() {
        Ok(cnf) => cnf,
        Err(_) => panic!("Cannot resolve configuration, exit.")
    }
}

async fn build_client() -> Client {
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::GUILD_MEMBERS | GatewayIntents::DIRECT_MESSAGES | GatewayIntents::MESSAGE_CONTENT;
    match Client::builder(&constants::TOKEN, intents).event_handler(Handler).await {
        Ok(client) => client,
        Err(err) => {
            utils::error("ClientBuilder", "cannot initialize client", err.to_string().as_str());
            panic!("Cannot initialize client, exit.")
        }
    }
}

fn copyright(){
    if !check_comp_id() {
        print!("\n    \x1b[35mMio Engine\x1b[0m\n");
        print!("       Created by \x1b[2mSedorriku#1949\x1b[0m with ❤️\n\n");
    }
}

#[tokio::main]
async fn main() {

    // Mio Engine
    copyright();
    utils::info("MioEngine", "initialisation...");

    // CONFIG 

    let config = get_config();
    if config.client.dev {
        std::env::set_var("RUST_BACKTRACE", "1");
        utils::info("DevMode", "This instance is initialised as in-dev.");
    } else {
        utils::info("ProdMode", "This instance is initialised as production-ready.");
        std::env::set_var("RUST_BACKTRACE", "0");
    }

    // MEFS
    #[allow(unused_mut)]
    let mut archive = {
        let arch = archive::Archive::from_file(
            &ARCHIVE_DIR.to_string(),
            config.client.version.clone(),
            config.security.rewrite_archive_if_invalid.clone(),
            config.security.auto_save_archive.clone()
        );
        match arch {
            Ok(a) => a,
            Err(err) => {
                utils::error("ArchiveSystem", "cannot load archive", err.as_str());
                if config.security.rewrite_archive_if_invalid {
                    archive::Archive::new(
                        &ARCHIVE_DIR.to_string(),
                        config.client.version.clone(),
                        config.security.rewrite_archive_if_invalid.clone(),
                        config.security.auto_save_archive.clone()
                    )
                } else {
                    utils::error("ArchiveSystem", "parameter `rewrite_archive_if_invalid` was disabled", "exit code 2");
                    exit(2)
                }
            }
        }
    };
    
    //let db_enc_key_temp = archive.get("DatabaseConnectionHandler", "db_key");
    //let db_enc_key = if db_enc_key_temp.is_null() { generate_encryption_key(512) } else { db_enc_key_temp.to_string() };
    //
    //let _ = archive.set("DatabaseConnectionLogging", "db_password", crate::libs::security::encryption::encrypt("Au2005az", db_enc_key.as_str()));
    //let _ = archive.set("DatabaseConnectionLogging", "db_user", crate::libs::security::encryption::encrypt("root", db_enc_key.as_str()));
    //let _ = archive.set("DatabaseConnectionLogging", "db_domain", crate::libs::security::encryption::encrypt("localhost", db_enc_key.as_str()));
    //let _ = archive.set("DatabaseConnectionLogging", "db_name", crate::libs::security::encryption::encrypt("mio", db_enc_key.as_str()));
    //let _ = archive.set("DatabaseConnectionLogging", "db_port", 3307);
    //let _ = archive.set("DatabaseConnectionLogging", "db_key", db_enc_key);

    utils::info("MioEngine", "loading mysql instance...");
    let mut conn = libs::database::create_database(&archive).await;
    
    let _q = sqlx::query("SELECT * FROM test WHERE id < 2;")
        .fetch_one(&mut conn)
        .await
        .unwrap();
    
    //dbg!(&q.get::<i32, _>("id"), &1);

    i18n::load(&config.i18n.locales_dir).await;
    i18n::test().await;


    let mut client = build_client().await;

    let stock: Storage = Storage::new(&config);
    {
        let mut data = client.data.write().await;
        data.insert::<Storage>(Arc::new(RwLock::new(stock)));
        data.insert::<Archive>(Arc::new(RwLock::new(archive)));
        drop(data);
    }

    // shard listener && ping manager
    let manager_copy = client.shard_manager.clone();
    let arc_client_data = client.data.clone();
    tokio::spawn(async move {
        let mut shards: HashMap<u64, Latency> = HashMap::new();
        loop {
            sleep(Duration::from_secs(10)).await;
    
            // manage all shards and free the ShardManager after
            {
                let lock = manager_copy.lock().await;
                let shard_runners = lock.runners.lock().await;
                for (id, runner) in shard_runners.iter() {
                    let ping = runner.latency.unwrap_or(Duration::from_millis(0));
                    // save every latency
                    if let Some(s) = shards.get_mut(&id.0) { s.ping = ping }
                    else { shards.insert(id.0, Latency { ping, warned: false }); };
                    
                    // warn if the ping is to high
                    if let Some(s) = shards.get_mut(&id.0) {
                        if ping.as_millis() > constants::SHARD_PING_WARN_MIN && !s.warned {
                            // AYO
                            utils::warn("ShardLatency", format!("The shard {} have a latency of {}ms, the ping is to high and may cause user-side latency", id.0, ping.as_millis()).as_str());
                            s.warned = true;
                            runner.runner_tx.set_status(serenity::model::user::OnlineStatus::DoNotDisturb);
                        } else {
                            if s.warned &&  ping.as_millis() < constants::SHARD_PING_WARN_MIN {
                                utils::info("ShardLatency", format!("The shard {} have a latency of {}ms, the ping is now normal", id.0, ping.as_millis()).as_str());
                                s.warned = false;
                                runner.runner_tx.set_status(serenity::model::user::OnlineStatus::Online);
                            }
                        }
                    }
                };
            }


            {
                let client_data = arc_client_data.write().await;
            
                // stock new latency in the Storage instance
                let storage_lock = client_data.get::<Storage>().expect("Expected Storage in TypeMap").clone();
                let mut storage = storage_lock.write().await;
                storage.latency = shards.clone();
            }
        }
    });

    // sigint handler
    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                println!("\n");
                utils::info("MioEngine", "Exit Signal received");
                let mut shards = shard_manager.lock().await;
                utils::info("MioEngine", format!("Shutting down all shards... ({} shard.s)", shards.shards_instantiated().await.len()).as_str());
                shards.shutdown_all().await;
                utils::success("MioEngine", "All shards have been killed");
                utils::success("MioEngine", "Exit code 0");
                std::process::exit(0);
            },
            Err(err) => {
                eprintln!("Unable to listen for shutdown signal: {}", err);
                std::process::exit(1);
            },
        };
    });

    // status
    let manager_copy = client.shard_manager.clone();
    let arc_client_data = client.data.clone();
    let status_time = Duration::from_secs(config.params.status_time.try_into().unwrap());
    tokio::spawn(async move {
        let mut status_index = 0;
        loop {
            sleep(status_time).await;
            
            let client_data = arc_client_data.read().await;
            let lock = manager_copy.lock().await;
            let shard_runners = lock.runners.lock().await;

            let storage_lock = client_data.get::<Storage>().expect("Expected Storage in TypeMap");
            let storage = storage_lock.read().await;

            {
                status_index = (status_index + 1) % storage.status.list.len();

                let new_state = if storage.dev { storage.status.dev_status.clone() }
                    else if storage.debug { storage.status.debug_mode_status.clone() }
                    else if storage.maintenance { storage.status.maintenance_status.clone() }
                    else { storage.status.list.get(status_index).unwrap_or(&Status { message: "👋".to_string(), status_type: storage::ClientActivityType::Watching }).clone() };
                    
                for (_id, runner) in shard_runners.iter() {
                    // for every shards
                    match new_state.status_type {
                        ClientActivityType::Playing => { runner.runner_tx.set_activity(Some(Activity::playing(new_state.message.clone()))); }
                        ClientActivityType::Watching => { runner.runner_tx.set_activity(Some(Activity::watching(new_state.message.clone()))); }
                        ClientActivityType::Listening => { runner.runner_tx.set_activity(Some(Activity::listening(new_state.message.clone()))); }
                        ClientActivityType::Streaming => { runner.runner_tx.set_activity(Some(Activity::streaming(new_state.message.clone(), storage.status.streaming_url.clone()))); }
                        ClientActivityType::Unknown => {}
                    };
                };
                if new_state.status_type.is_unknown() { utils::warn("StatusLoop", "Status type was unknown"); }
            }
        }
    });

    
    // before login time trace
    {
        let storage_lock = {
            let data = client.data.read().await;
            data.get::<Storage>().expect("Expected Storage in TypeMap.").clone()
        };
        let storage = storage_lock.read().await;

        let now = Utc::now();
        let start_time = now.timestamp_micros() - storage.process_start.timestamp_micros();
        utils::info(
            "MioEngine",
            format!("System initialized in {}ms ({start_time}µs)", start_time / 1000).as_str()
        );
    }

    if let Err(why) = client.start().await {
        utils::error("ClientStarter", "cannot start the client", why.to_string().as_str());
        panic!("Cannot start the client, exit.")
    }
}