use anyhow::{Context as _, Result};
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

mod errors;
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
    token: String,
    prefix: String,
    allowed_channels: HashMap<String, ChannelId>,
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
    let config = Config::read_from_file("config.toml")?;
    let handler = Handler;

    let mut client = Client::new(&config.token, handler).context("Couldn't create client")?;
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
        .normal_message(|ctx, msg| {
            if !msg.is_private() || msg.author.bot {
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
        })
        .group(&IOGAME_GROUP));
    client.data.write().insert::<Config>(config);
    client.start()?;
    Ok(())
}

#[command]
fn ping(ctx: &mut Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "pong!")?;

    Ok(())
}

#[group("iogame")]
#[commands(ping)]
struct IOGame;
