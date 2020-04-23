use anyhow::{Context as _, Result};
use serde::Deserialize;
use serenity::prelude::*;
use serenity::model::prelude::*;
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
    prefix: String,
}

impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        if msg.content.starts_with(&self.prefix)
                && (msg.is_private() || self.allowed_channels.contains(&msg.channel_id)) {
            dbg!(msg);
        }
    }
}

fn main() -> Result<()> {
    let config = Config::read_from_file("config.toml")?;
    let handler = Handler {
        allowed_channels: config.allowed_channels.values().copied().collect(),
        prefix: config.prefix,
    };

    let mut client = Client::new(&config.token, handler).context("Couldn't create client")?;
    client.start()?;
    Ok(())
}
