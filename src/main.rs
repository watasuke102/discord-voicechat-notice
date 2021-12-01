use serde::{Deserialize, Serialize};
use serenity::{
    async_trait,
    framework::StandardFramework,
    model::{gateway::Ready, id::ChannelId, id::GuildId, voice::VoiceState},
    prelude::*,
    utils::MessageBuilder,
};
use std::{fs::File, io::BufReader, option::Option, sync::Arc};

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!(
            "Info: connected {} [version: {}]",
            ready.user.name, ready.version
        );
    }
    async fn voice_state_update(
        &self,
        ctx: Context,
        _guild_id: Option<GuildId>,
        old: Option<VoiceState>,
        new: VoiceState,
    ) {
        let msg = MessageBuilder::new()
            .push(if let Some(u) = &new.member {
                &u.user.name
            } else {
                "someone"
            })
            // oldがSomeならLeave、NoneであればJoin
            .push(if let Some(_) = old {
                " leaved "
            } else {
                " joined "
            })
            // チャンネルIDがなかったら0にする（いいのかな）
            .channel(if let Some(c) = new.channel_id {
                c
            } else {
                ChannelId(0)
            })
            .build();
        let data = ctx.data.read().await;
        if let Some(id) = data.get::<LogChannelId>() {
            match id.clone().parse() {
                Ok(id) => {
                    let ch = ChannelId(id);
                    if let Err(e) = ch.say(&ctx.http, msg).await {
                        println!("ERROR: failed to send an message => {}", e);
                    }
                }
                Err(e) => println!("ERROR: failed to parse channel ID => {}", e),
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Settings {
    discord_token: String,
    log_channel_id: String,
}

struct LogChannelId;
impl TypeMapKey for LogChannelId {
    type Value = Arc<String>;
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
        if settings.log_channel_id == empty {
            panic!("ERROR: `log_channel_id` in env.json is empty");
        }
    }
    // クライアント生成と設定の読み込み
    let mut client = Client::builder(&settings.discord_token)
        .event_handler(Handler)
        .framework(StandardFramework::new())
        .await
        .expect("ERROR: failed to create client");
    {
        let mut data = client.data.write().await;
        data.insert::<LogChannelId>(Arc::new(settings.log_channel_id));
    }

    if let Err(why) = client.start().await {
        println!("ERROR: client error: {:#?}", why);
    }
}
