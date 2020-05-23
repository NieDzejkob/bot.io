use anyhow::{Context as _, Result};
use diesel::prelude::*;
use serde::Deserialize;
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    StandardFramework,
    CommandResult,
    macros::{command, group},
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[macro_use]
extern crate diesel;

mod db;
mod errors;
mod schema;
mod models;
use errors::MathError;

trait ErrorExt {
    fn log_error(&self);
}

impl ErrorExt for Result<()> {
    fn log_error(&self) {
        if let Err(why) = self {
            eprintln!("An error occured: {}", why);
        }
    }
}

#[derive(Deserialize)]
struct Config {
    prefix: String,
    #[serde(default)]
    database: db::DatabaseConfig,
    allowed_channels: HashMap<String, ChannelId>,
    admin_users: HashMap<String, UserId>,
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

impl EventHandler for Handler {}

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
                eprintln!("Message {:?} triggered an error: {:?}", msg.content, why);
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

            let command = mathparser::parse_command(&msg.content);
            use mathparser::Command;
            match command {
                Ok(Command::Expr(e)) => {
                    dbg!(e);
                }
                Err(why) => {
                    let error: MathError = why.into();
                    error.send_to_user(ctx, &msg.author, &msg.content).log_error();
                }
                _ => (),
            }
        }})
        .group(&IOGAME_GROUP));
    client.data.write().insert::<Config>(config);
    client.data.write().insert::<db::DB>(db);
    client.start()?;
    Ok(())
}

#[command]
fn problems(ctx: &mut Context, msg: &Message) -> CommandResult {
    use schema::problems::dsl::*;

    let results = problems.load::<models::Problem>(&db::get_connection(ctx)?)
        .context("Fetch problems from database")?;
    msg.author.dm(ctx, |m| m.content(format!("{} problems available", results.len())))?;

    Ok(())
}

#[group("iogame")]
#[commands(problems)]
struct IOGame;
