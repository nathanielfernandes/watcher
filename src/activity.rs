use serde::{Deserialize, Serialize};
use serenity::model::prelude::Activity;
use specta::Type;

use crate::detectable::{Detectable, DETECABLES};

#[derive(Type, Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DiscordActivity {
    Activity { activity: BaseActivity },
    Spotify { activity: SpotifyActivity },
}

impl From<Activity> for DiscordActivity {
    fn from(activity: Activity) -> Self {
        if activity.name == "Spotify" {
            return DiscordActivity::Spotify {
                activity: activity.into(),
            };
        }
        DiscordActivity::Activity {
            activity: activity.into(),
        }
    }
}

#[derive(Type, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assets {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub small_text: Option<String>,
}

#[allow(non_camel_case_types)]
#[derive(Type, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActivityType {
    playing,
    streaming,
    listening,
    watching,
    custom,
    unknown,
}

#[derive(Type, Debug, Clone, Serialize, Deserialize)]
pub struct BaseActivity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets: Option<Assets>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,

    #[serde(rename = "type")]
    pub kind: ActivityType,

    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub detectable: Option<Detectable>,
}

fn fix_url(url: &str, app_id: &Option<String>) -> Option<String> {
    if url.starts_with("mp:") {
        return Some(format!("https://media.discordapp.net/{}", &url[3..]));
    }

    if let Some(app_id) = app_id {
        return Some(format!(
            "https://cdn.discordapp.com/app-assets/{}/{}.png",
            app_id, url
        ));
    }

    None
}

impl From<Activity> for BaseActivity {
    fn from(activity: Activity) -> Self {
        let application_id = activity.application_id.map(|id| id.0.to_string());

        let assets = activity.assets.map(|assets| Assets {
            large_image: assets
                .large_image
                .and_then(|url| fix_url(&url, &application_id)),
            large_text: assets.large_text,
            small_image: assets
                .small_image
                .and_then(|url| fix_url(&url, &application_id)),
            small_text: assets.small_text,
        });

        let details = activity.details;

        let (start_time, end_time) = activity
            .timestamps
            .map(|timestamps| (timestamps.start, timestamps.end))
            .unwrap_or_default();

        let kind = match activity.kind {
            serenity::model::prelude::ActivityType::Playing => ActivityType::playing,
            serenity::model::prelude::ActivityType::Streaming => ActivityType::streaming,
            serenity::model::prelude::ActivityType::Listening => ActivityType::listening,
            serenity::model::prelude::ActivityType::Watching => ActivityType::watching,
            serenity::model::prelude::ActivityType::Custom => ActivityType::custom,
            _ => ActivityType::unknown,
        };

        let detectable = DETECABLES.get(&activity.name).cloned();
        let name = activity.name;

        let state = activity.state;

        BaseActivity {
            application_id,
            assets,
            details,
            kind,
            name,
            state,
            start_time: start_time.map(|time| time as u32),
            end_time: end_time.map(|time| time as u32),
            detectable,
        }
    }
}

#[derive(Type, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpotifyActivity {
    pub album: String,
    pub album_cover_url: String,
    pub share_cover_url: String,
    pub artist: String,
    pub artists: Vec<String>,
    pub title: String,
    pub track_id: String,
    pub track_url: String,
    pub start: u32,
    pub end: u32,
    pub duration: u32,
}

impl From<Activity> for SpotifyActivity {
    fn from(activity: Activity) -> Self {
        let album = activity
            .assets
            .as_ref()
            .and_then(|assets| assets.large_text.clone())
            .unwrap_or_default();
        let album_cover_url = activity
            .assets
            .and_then(|assets| assets.large_image)
            .map(|url| {
                if !url.starts_with("spotify:") {
                    return "".to_string();
                };

                format!("https://i.scdn.co/image/{}", &url[8..])
            })
            .unwrap_or_default();
        let artist = activity.state.clone().unwrap_or_default();
        let artists = activity
            .state
            .clone()
            .unwrap_or_default()
            .split(';')
            .map(|artist| artist.trim().to_string())
            .collect();

        let title = activity.details.unwrap_or_default();
        let track_id = activity.sync_id.unwrap_or_default();
        let share_cover_url = format!("https://spv.ncp.nathanferns.xyz/{}", &track_id);
        let track_url = format!("https://open.spotify.com/track/{}", &track_id);
        let start = activity
            .timestamps
            .as_ref()
            .and_then(|timestamps| timestamps.start)
            .unwrap_or_default() as u32;
        let end = activity
            .timestamps
            .and_then(|timestamps| timestamps.end)
            .unwrap_or_default() as u32;
        let duration = end - start;

        SpotifyActivity {
            album,
            album_cover_url,
            share_cover_url,
            artist,
            artists,
            title,
            track_id,
            track_url,
            start,
            end,
            duration,
        }
    }
}

pub trait NoCopy {
    fn imstr(&self) -> imstr::ImString;
}

impl NoCopy for Vec<DiscordActivity> {
    fn imstr(&self) -> imstr::ImString {
        let string = serde_json::to_string_pretty(self).unwrap_or_default();
        imstr::ImString::from(string)
    }
}
