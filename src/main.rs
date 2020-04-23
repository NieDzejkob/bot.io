use anyhow::{Context as _, Result};
use serde::Deserialize;
use serenity::prelude::*;
use serenity::model::prelude::*;
use serenity::framework::standard::{
    StandardFramework,
    CommandResult,
    macros::{command, group},
};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

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

struct Handler {
    allowed_channels: HashSet<ChannelId>,
}

impl EventHandler for Handler {}

fn main() -> Result<()> {
    let config = Config::read_from_file("config.toml")?;
    let handler = Handler {
        allowed_channels: config.allowed_channels.values().copied().collect(),
    };

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
            dbg!(command);
        })
        .group(&IOGAME_GROUP));
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
