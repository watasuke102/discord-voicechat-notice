// discord-voicechat-notice
// main.rs
//
// CopyRight (c) 2021 Watasuke
// Email  : <watasuke102@gmail.com>
// Twitter: @Watasuke102
// This software is released under the MIT SUSHI-WARE License.
use chrono::FixedOffset;
use serde::{Deserialize, Serialize};
use serenity::{
    async_trait,
    builder::{CreateEmbed, CreateMessage},
    framework::StandardFramework,
    model::{gateway::Ready, id::ChannelId, voice::VoiceState, Timestamp},
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
    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        let data = ctx.data.read().await;
        if let Some(data) = data.get::<Settings>() {
            let Some(guild_id) = new.guild_id else {
                return;
            };
            // 設定に記載されたサーバーと違ったら終わり
            let data = data.clone();
            if guild_id != data.guild_id {
                return;
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
            // leaveの場合はold、Joinの場合はnewからチャンネルIDを得る
            let (status, channel_id) = if let Some(old) = old {
                if let None = &new.channel_id {
                    (Status::Leaved, old.channel_id)
                } else {
                    (Status::Other, old.channel_id)
                }
            } else {
                (Status::Joined, new.channel_id)
            };
            // ミュートされただけ等であれば終わり
            if status == Status::Other {
                return;
            }
            // チャンネル名を取得
            let unknown_message = "Unknown channel".to_string();
            let channel_name = if let Some(id) = channel_id {
                let channel_name = id.name(ctx.cache.as_ref()).await;
                channel_name.unwrap_or(unknown_message)
            } else {
                unknown_message
            };
            // ユーザー名を取得、ニックネームが使用不可ならユーザー名
            // FIXME: display_nameがユーザーIDになってしまう　上流（nextブランチ）のrelease待ち
            let user_name = if let Some(u) = &new.member {
                if let Some(nick_name) = u.user.nick_in(&ctx, guild_id).await {
                    nick_name.clone()
                } else {
                    u.display_name().to_string()
                }
            } else {
                "Unknown user".to_string()
            };
            // メッセージをビルド・送信
            let ch = ChannelId(data.log_channel_id);
            if let Err(e) = ch
                .send_message(&ctx.http, |m: &mut CreateMessage| {
                    // メッセージ作成
                    m.embed(|e: &mut CreateEmbed| {
                        e.title("Voice Channel Notice");
                        if let Some(u) = &new.member {
                            if status == Status::Joined {
                                e.description(format!("**{}** joined VC", user_name));
                                e.color(Colour(0x2aed24));
                            } else {
                                e.description(format!("**{}** leaved VC", user_name));
                                e.color(Colour(0xed2424));
                            }
                            // アバターの設定
                            if let Some(url) = &u.user.avatar_url() {
                                e.thumbnail(url);
                            }
                        }
                        e.field("Channel", channel_name, false);
                        // タイムゾーンをJSTに変更した現在時刻をタイムスタンプに使用
                        e.timestamp(
                            Timestamp::now()
                                .with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap()),
                        );
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
    let mut client = Client::builder(
        &settings.discord_token,
        GatewayIntents::GUILDS | GatewayIntents::GUILD_VOICE_STATES,
    )
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
