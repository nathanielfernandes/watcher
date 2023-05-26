pub mod activity;

use crate::activity::Handler;

use async_stream::stream;
use axum::{
    body::Body,
    extract::Path,
    response::{
        sse::{Event, KeepAlive},
        Response, Sse,
    },
    routing::get,
    Json, Router,
};
use futures::Stream;
use serenity::{model::gateway::GatewayIntents, Client};
use std::{convert::Infallible, env, net::SocketAddr};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    println!("Starting up...");

    tokio::spawn(async move {
        let token = env::var("TOKEN").expect("Expected a token in the environment");

        let intents = GatewayIntents::GUILD_PRESENCES | GatewayIntents::GUILD_MEMBERS;

        let mut client = Client::builder(token, intents)
            .event_handler(Handler)
            .await
            .expect("Error creating client");

        if let Err(why) = client.start().await {
            println!("Client error: {:?}", why);
        }
    });

    let cors = CorsLayer::very_permissive();

    let app = Router::new()
        .route("/", get(root))
        .route("/activity/:user_id", get(get_activity))
        .route("/live-activity/:user_id", get(live_activity))
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 2223));

    println!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn root() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    Sse::new(stream! {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            yield Ok(Event::default().data("Hello, world!"));
        }
    })
    .keep_alive(KeepAlive::new().interval(std::time::Duration::from_secs(10)))
}

async fn get_activity(Path(user_id): Path<u64>) -> Json<Vec<activity::DiscordActivity>> {
    Json(activity::get_activity(user_id))
}

async fn live_activity(
    Path(user_id): Path<u64>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, Response<Body>> {
    if let Some(rx) = activity::get_live_activity(user_id) {
        Ok(Sse::new(stream! {
            let mut last = None;

            loop {
                match rx.recv().await {
                    Ok(activity) => {
                        if last.is_none() || last.unwrap() != activity {
                            yield Ok(Event::default().json_data(activity.clone()).expect("failed to serialize json"));
                        }
                        last = Some(activity);
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        })
        .keep_alive(KeepAlive::new().interval(std::time::Duration::from_secs(10))))
    } else {
        Err(Response::builder()
            .status(400)
            .body(Body::from("User not in allow list"))
            .unwrap())
    }
}
