use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;


#[derive(Serialize, Deserialize, Debug)]
struct Settings {
    discord_token: String,
    voice_channel_id: String,
    log_channel_id: String,
}

fn main() {
    let file = File::open("env.json")
        .expect("cannot read `env.json`: did you create this file? try `cp sample-env.json env.json` and edit it.");
    let settings: Settings = serde_json::from_reader(BufReader::new(file)).unwrap();
    {
        let empty = String::new();
        if settings.discord_token == empty {
            panic!("FATAL: `discord_token` in env.json is empty");
        }
        if settings.voice_channel_id == empty {
            panic!("FATAL: `voice_id` in env.json is empty");
        }
        if settings.log_channel_id == empty {
            panic!("FATAL: `log_channel_id` in env.json is empty");
        }
    }
    println!("{:?}", settings);
}
