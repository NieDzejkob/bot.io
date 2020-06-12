use serde::Deserialize;
use serenity::framework::standard::{
    StandardFramework,
    DispatchError, Reason,
    macros::group,
};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::Arc;
use std::path::Path;

#[macro_use] extern crate diesel;
#[macro_use] extern crate extension_trait;
#[macro_use] extern crate ifmt;
#[macro_use] extern crate rental;
#[macro_use] extern crate strum_macros;

pub mod admin;
pub mod config;
pub mod db;
pub mod errors;
pub mod eval;
pub mod schema;
pub mod models;
pub mod interactive;
pub mod problem;
pub mod reactions;

/// Re-exports types commonly used in the codebase.
///
/// Feel free to wildcard-import it with `use crate::prelude::*;`.
pub mod prelude {
    pub use anyhow::{Context as _, Result};
    pub use genawaiter::sync::Gen;
    pub use serenity::prelude::*;
    pub use serenity::model::prelude::*;
    pub use serenity::framework::standard::{
        Args, CommandResult,
        macros::command,
    };
    pub use serenity::utils::{Color, MessageBuilder};

    pub use crate::interactive::InteractiveCommand;
    pub use crate::ErrorExt;
    pub use crate::errors::MathErrorExt;
}

use prelude::*;

#[extension_trait(pub)]
impl<T> ErrorExt for Result<T> {
    fn log_error(&self) {
        if let Err(why) = self {
            log::error!("An error occured: {}", why);
        }
    }
}

#[derive(Deserialize)]
struct Config {
    prefix: String,
    #[serde(default)]
    database: db::DatabaseConfig,
    #[serde(deserialize_with="config::id_list")]
    admin_users: HashSet<UserId>,
}

impl Config {
    fn read_from_file(file: impl AsRef<Path>) -> Result<Self> {
        let file = file.as_ref();
        let config = fs::read_to_string(file)
            .with_context(|| format!("Failed to read configuration file {:?}", file))?;
        toml::de::from_str(&config)
            .context("Failed to parse configuration file")
    }
}

impl TypeMapKey for Config {
    type Value = Config;
}

struct Handler;

impl EventHandler for Handler {
    fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        interactive::handle_reaction(&ctx, reaction);
    }

    fn reaction_remove(&self, ctx: Context, reaction: Reaction) {
        interactive::handle_reaction(&ctx, reaction);
    }
}

fn main() -> Result<()> {
    dotenv::dotenv().ok();
    let token = dotenv::var("DISCORD_TOKEN").context("DISCORD_TOKEN must be set")?;

    env_logger::init();
    let config = Config::read_from_file("config.toml")?;
    let db = db::connect(&config.database)?;
    log::info!("Connected to database");
    let handler = Handler;

    let mut client = Client::new(token, handler).context("Couldn't create client")?;
    client.with_framework(
        StandardFramework::new()
        .configure(|c| c
            .prefix(&config.prefix))
        .after(|ctx, msg, _, result| {
            let result = result.and_then(|()| if !msg.is_private() {
                msg.react(&ctx, '👌').context("add an OK reaction").map_err(From::from)
            } else { Ok(()) });

            if let Err(why) = result {
                log::error!("Message {:?} triggered an error: {:?}", msg.content, why);
            }
        })
        .unrecognised_command(|_ctx, _msg, cmd| {
            // TODO: suggest a similar command
            log::warn!("Unrecognized command: {}", cmd);
        })
        .normal_message({
            let prefix = config.prefix.clone();
        move |ctx, msg| {
            if !msg.is_private() || msg.author.bot
                || msg.content.starts_with(&prefix) {
                return;
            }

            if let Err(why) = crate::interactive::handle_message(ctx, msg) {
                log::error!("Message {:?} triggered an error: {:?}", msg.content, why);
            }
        }})
        .on_dispatch_error(|ctx, msg, why| {
            match why {
                DispatchError::CheckFailed(_, Reason::User(reason)) => {
                    msg.reply(&ctx, reason)
                        .context("Send permission error message").log_error();
                }
                _ => {}
            }
        })
        .group(&IOGAME_GROUP)
        .group(&admin::ADMIN_GROUP));
    client.data.write().insert::<Config>(config);
    client.data.write().insert::<db::DB>(db);
    client.data.write().insert::<interactive::InteractionStates>(HashMap::new());

    let shard_manager = Arc::clone(&client.shard_manager);
    ctrlc::set_handler(move || {
        log::info!("Shutting down...");
        shard_manager.lock().shutdown_all();
    }).context("Setting the Ctrl-C handler")?;

    client.start()?;
    Ok(())
}

#[group]
#[commands(problems)]
struct IOGame;
use problem::PROBLEMS_COMMAND;
