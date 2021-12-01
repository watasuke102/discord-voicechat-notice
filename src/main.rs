use serde::{Deserialize, Serialize};
use serenity::{
    async_trait,
    builder::{CreateEmbed, CreateMessage},
    framework::StandardFramework,
    model::{gateway::Ready, id::ChannelId, id::GuildId, voice::VoiceState},
    prelude::*,
    utils::Colour,
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
        println!("{:?}", ctx.http);
        let data = ctx.data.read().await;
        if let Some(id) = data.get::<LogChannelId>() {
            match id.clone().parse() {
                Ok(id) => {
                    let ch = ChannelId(id);
                    let channel_name = ch.name(ctx.cache.as_ref()).await;
                    let channel_name = channel_name.unwrap_or("Unknown channel".to_string());
                    if let Err(e) = ch
                        .send_message(&ctx.http, |m: &mut CreateMessage| {
                            // メッセージ作成
                            m.embed(|e: &mut CreateEmbed| {
                                e.title("Voice Channel Notice");
                                // oldがSomeならLeave、NoneであればJoin
                                if let Some(_) = old {
                                    e.description("Someone leaved VC");
                                    e.color(Colour(0xed2424));
                                } else {
                                    e.description("Someone joined VC");
                                    e.color(Colour(0x2aed24));
                                }
                                // アバターの設定
                                if let Some(u) = &new.member {
                                    e.field(
                                        &u.user.name,
                                        format!("Channel: {}", channel_name),
                                        false,
                                    );
                                    if let Some(avatar) = &u.user.avatar {
                                        let url = format!(
                                            "https://cdn.discordapp.com/avatars/{}/{}.webp",
                                            u.user.id, avatar
                                        );
                                        e.thumbnail(url);
                                    }
                                } else {
                                }
                                e
                            });
                            m
                        })
                        .await
                    {
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
