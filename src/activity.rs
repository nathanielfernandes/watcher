use async_channel::{Receiver, Sender};
use moka::future::Cache;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serenity::async_trait;
use serenity::model::prelude::{Activity, ActivityType, Presence, Ready};
use serenity::prelude::*;
use std::collections::HashMap;
use std::env;

const SPOTIFY_CDN: &'static str = "https://i.scdn.co/image/";
static ALLOW_LIST: Lazy<Vec<u64>> = Lazy::new(|| {
    env::var("ALLOW_LIST")
        .expect("Expected a list of user IDs in the environment")
        .split(',')
        .map(|id| {
            id.parse::<u64>()
                .expect("Expected a list of user IDs in the environment")
        })
        .collect()
});

static CACHE: Lazy<Cache<u64, Vec<DiscordActivity>>> = Lazy::new(|| {
    Cache::builder()
        .time_to_live(std::time::Duration::from_secs(3600))
        .build()
});

static CHANNELS: Lazy<(
    HashMap<u64, Receiver<Vec<DiscordActivity>>>,
    HashMap<u64, Sender<Vec<DiscordActivity>>>,
)> = Lazy::new(|| {
    let mut recievers = HashMap::new();
    let mut senders = HashMap::new();

    for user_id in ALLOW_LIST.iter() {
        let (sender, reciever) = async_channel::unbounded();
        senders.insert(*user_id, sender);
        recievers.insert(*user_id, reciever);
    }

    (recievers, senders)
});

pub fn get_activity(user_id: u64) -> Vec<DiscordActivity> {
    CACHE.get(&user_id).unwrap_or_default()
}

pub fn get_live_activity(user_id: u64) -> Option<Receiver<Vec<DiscordActivity>>> {
    CHANNELS.0.get(&user_id).cloned()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscordActivity {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: &'static str,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hover_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_url: Option<String>,

    pub start_time: Option<u64>,
    pub end_time: Option<u64>,
}

impl From<Activity> for DiscordActivity {
    fn from(activity: Activity) -> Self {
        let asset_url = activity
            .assets
            .clone()
            .and_then(|asset| asset.large_image.or(asset.small_image).map(|url| url))
            .map(|url| {
                if url.starts_with("spotify:") {
                    let id = url.split(':').last().unwrap_or_default();
                    format!("{}{}", SPOTIFY_CDN, id)
                } else if url.starts_with("twitch:") {
                    let id = url.split('/').last().unwrap_or_default();
                    format!(
                        "https://static-cdn.jtvnw.net/previews-ttv/live_user_{}.jpg",
                        id
                    )
                } else if url.starts_with("mp:external") {
                    let id = url.split("https/").last().unwrap_or_default();
                    format!("https://{}", id)
                } else {
                    let app_id = activity.application_id.unwrap_or_default();
                    format!(
                        "https://cdn.discordapp.com/app-assets/{}/{}.png",
                        app_id, url
                    )
                }
            });

        let asset_text = activity
            .assets
            .and_then(|asset| asset.large_text.or(asset.small_text).map(|text| text));

        let (start_time, end_time) = activity
            .timestamps
            .map(|timestamps| (timestamps.start, timestamps.end))
            .unwrap_or_default();

        DiscordActivity {
            name: activity.name,
            details: activity.details,

            kind: match activity.kind {
                ActivityType::Playing => "playing",
                ActivityType::Streaming => "streaming",
                ActivityType::Listening => "listening",
                ActivityType::Watching => "watching",
                ActivityType::Custom => "custom",
                _ => "unknown",
            },

            hover_text: asset_text,
            asset_url,

            start_time,
            end_time,
        }
    }
}

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // As the intents set in this example, this event shall never be dispatched.
    // Try it by changing your status.
    async fn presence_update(&self, _ctx: Context, presence: Presence) {
        const SPOTIFY_CDN: &'static str = "https://i.scdn.co/image/";

        let user_id = presence.user.id.0;

        if let Some(tx) = CHANNELS.1.get(&user_id) {
            let activites = presence
                .activities
                .into_iter()
                .map(|a| a.into())
                .collect::<Vec<DiscordActivity>>();
            CACHE.insert(user_id, activites.clone()).await;

            if let Err(_) = tx.send(activites).await {
                println!("Failed to send presence update to {}", user_id);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}
