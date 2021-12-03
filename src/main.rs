// discord-voicechat-notice
// main.rs
//
// CopyRight (c) 2021 Watasuke
// Email  : <watasuke102@gmail.com>
// Twitter: @Watasuke102
// This software is released under the MIT SUSHI-WARE License.
use chrono::{FixedOffset, Utc};
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
        guild_id: Option<GuildId>,
        old: Option<VoiceState>,
        new: VoiceState,
    ) {
        let data = ctx.data.read().await;
        if let Some(data) = data.get::<Settings>() {
            let data = data.clone();
            // 設定に記載されたサーバーと違ったら終わり
            if let Some(id) = guild_id {
                if id != data.guild_id {
                    return;
                }
            }
            // 入退出の判定
            #[derive(PartialEq)]
            enum Status {
                Joined,
                Leaved,
                Other,
            }
            // oldがSomeかつnewのchannel_idがNoneであればLeave
            // oldがNoneであればJoin
            let status = if let Some(_) = old {
                if let None = &new.channel_id {
                    Status::Leaved
                } else {
                    Status::Other
                }
            } else {
                Status::Joined
            };
            // ミュートされただけ等であれば終わり
            if status == Status::Other {
                return;
            }
            // チャンネル名の取得
            let unknown_message = "Unknown channel".to_string();
            let channel_name = if let Some(id) = new.channel_id {
                let channel_name = id.name(ctx.cache.as_ref()).await;
                channel_name.unwrap_or(unknown_message)
            } else {
                unknown_message
            };
            // メッセージをビルド・送信
            let ch = ChannelId(data.log_channel_id);
            if let Err(e) = ch
                .send_message(&ctx.http, |m: &mut CreateMessage| {
                    // メッセージ作成
                    m.embed(|e: &mut CreateEmbed| {
                        e.title("Voice Channel Notice");
                        if status == Status::Joined {
                            e.description("Someone joined VC");
                            e.color(Colour(0x2aed24));
                        } else {
                            e.description("Someone leaved VC");
                            e.color(Colour(0xed2424));
                        }
                        // アバターの設定
                        if let Some(u) = &new.member {
                            e.field(&u.user.name, format!("Channel: {}", channel_name), false);
                            if let Some(avatar) = &u.user.avatar {
                                let url = format!(
                                    "https://cdn.discordapp.com/avatars/{}/{}.webp",
                                    u.user.id, avatar
                                );
                                e.thumbnail(url);
                            }
                        } else {
                            e.field("Unknown user", format!("Channel: {}", channel_name), false);
                        }
                        e.timestamp(&Utc::now().with_timezone(&FixedOffset::east(9 * 3600)));
                        e
                    });
                    m
                })
                .await
            {
                println!("ERROR: failed to send an message => {}", e);
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Settings {
    discord_token: String,
    guild_id: u64,
    log_channel_id: u64,
}

impl TypeMapKey for Settings {
    type Value = Arc<Settings>;
}

#[tokio::main]
async fn main() {
    let file = File::open("env.json")
        .expect("cannot read `env.json`: did you create this file? try `cp sample-env.json env.json` and edit it.");
    let settings: Settings = serde_json::from_reader(BufReader::new(file)).unwrap();
    {
        if settings.discord_token == String::new() {
            panic!("ERROR: `discord_token` in env.json is empty");
        }
        if settings.guild_id == 0 {
            panic!("ERROR: `guild_id` in env.json is empty");
        }
        if settings.log_channel_id == 0 {
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
        data.insert::<Settings>(Arc::new(settings));
    }

    if let Err(why) = client.start().await {
        println!("ERROR: client error: {:#?}", why);
    }
}
