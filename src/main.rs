use serde::{Deserialize, Serialize};
use serenity::framework::StandardFramework;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, id::GuildId, voice::VoiceState},
    prelude::*,
};
use std::fs::File;
use std::io::BufReader;
use std::option::Option;

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, _ctx: Context, _new_message: Message) {
        println!("{:?}", _new_message);
    }
    async fn ready(&self, _: Context, ready: Ready) {
        println!(
            "Info: connected {} [version: {}]",
            ready.user.name, ready.version
        );
    }
    async fn voice_state_update(
        &self,
        _ctx: Context,
        guild_id: Option<GuildId>,
        _: Option<VoiceState>,
        new: VoiceState,
    ) {
        println!("*********************");
        println!("guild_id: {:?}", guild_id);
        if let Some(n) = &new.member {
            println!("[new] {:#?}", n.user);
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Settings {
    discord_token: String,
    voice_channel_id: String,
    log_channel_id: String,
}

#[tokio::main]
async fn main() {
    let file = File::open("env.json")
        .expect("cannot read `env.json`: did you create this file? try `cp sample-env.json env.json` and edit it.");
    let settings: Settings = serde_json::from_reader(BufReader::new(file)).unwrap();
    {
        let empty = String::new();
        if settings.discord_token == empty {
            panic!("ERROR: `discord_token` in env.json is empty");
        }
        if settings.voice_channel_id == empty {
            panic!("ERROR: `voice_id` in env.json is empty");
        }
        if settings.log_channel_id == empty {
            panic!("ERROR: `log_channel_id` in env.json is empty");
        }
    }
    let mut client = Client::builder(&settings.discord_token)
        .event_handler(Handler)
        .framework(StandardFramework::new())
        .await
        .expect("ERROR: failed to create client");
    if let Err(why) = client.start().await {
        println!("ERROR: client error: {:#?}", why);
    }
}
