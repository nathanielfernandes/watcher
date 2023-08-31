use serde::{Deserialize, Serialize};
use serenity::model::prelude::{Activity, ActivityType};
use specta::Type;

const SPOTIFY_CDN: &'static str = "https://i.scdn.co/image/";

#[derive(Type, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiscordActivity {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: &'static str,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hover_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<u32>,
}

impl DiscordActivity {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
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
            state: activity.state,

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

            start_time: start_time.map(|time| time as u32),
            end_time: end_time.map(|time| time as u32),
        }
    }
}

pub trait NoCopy {
    fn imstr(&self) -> imstr::ImString;
}

impl NoCopy for DiscordActivity {
    fn imstr(&self) -> imstr::ImString {
        imstr::ImString::from(self.to_json().as_str())
    }
}

impl NoCopy for Vec<DiscordActivity> {
    fn imstr(&self) -> imstr::ImString {
        imstr::ImString::from(serde_json::to_string(self).unwrap_or_default().as_str())
    }
}
