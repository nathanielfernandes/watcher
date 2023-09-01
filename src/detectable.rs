use specta::Type;
use std::collections::HashMap;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
struct Game {
    id: String,
    name: String,
    description: String,

    icon: Option<String>,
    splash: Option<String>,
    cover_image: Option<String>,

    publishers: Option<Vec<Author>>,
    developers: Option<Vec<Author>>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Author {
    name: String,
}

#[derive(Type, Clone, Debug, Serialize, Deserialize)]
pub struct Detectable {
    description: String,
    icon_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    cover_url: Option<String>,
    splash_url: Option<String>,

    publishers: Vec<String>,
    developers: Vec<String>,
}

fn format_url(id: &str, icon: &str) -> String {
    format!("https://cdn.discordapp.com/app-icons/{}/{}.webp", id, icon)
}

pub static DETECABLES: Lazy<HashMap<String, Detectable>> = Lazy::new(|| {
    let resp: Vec<Game> =
        reqwest::blocking::get("https://discord.com/api/v9/applications/detectable")
            .expect("failed to get detecables.json")
            .json()
            .expect("failed to parse detecables.json");

    println!("got {} detecables", resp.len());

    let mut map = HashMap::with_capacity(resp.len());

    for game in resp {
        map.insert(
            game.name,
            Detectable {
                description: game.description,
                icon_url: game.icon.map(|icon| format_url(&game.id, &icon)),
                cover_url: game.cover_image.map(|cover| format_url(&game.id, &cover)),
                splash_url: game.splash.map(|splash| format_url(&game.id, &splash)),
                publishers: game
                    .publishers
                    .map(|p| p.into_iter().map(|p| p.name).collect())
                    .unwrap_or_default(),

                developers: game
                    .developers
                    .map(|p| p.into_iter().map(|p| p.name).collect())
                    .unwrap_or_default(),
            },
        );
    }

    map
});
