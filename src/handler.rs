use std::env;

use crate::{
    redis::{get_con, set_str},
    ws, Client, Clients, Result, pubsub,
};

use redis::{aio::Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use warp::{http::StatusCode, reply::json, ws::Message, Reply};

/*
Request body for register handler
*/
#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    user_id: usize,
    game_id: usize,
}

/*
Response body for register handler
*/
#[derive(Serialize, Debug)]
pub struct RegisterResponse {
    url: String,
}

/*
Request body for Make Move handler
*/
#[derive(Serialize, Debug)]
pub struct MakeMoveRequest {
    san: String,
    fen: String,
}

/*
Response body for Make Move handler
*/
#[derive(Serialize, Debug)]
pub struct MakeMoveResponse {
    move_made: bool,
}

/*
Body for Event
*/
#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    game_id: usize,
    user_id: Option<usize>,
    message: String,
}

pub async fn broadcast_handler(body: Event, clients: Clients) -> Result<impl Reply> {
    // Broadcast to every client that isn't the sender in the same game
    clients
        .read()
        .await
        .iter()
        .filter(|(_, client)| match body.user_id {
            Some(v) => client.user_id == v,
            None => true,
        })
        .filter(|(_, client)| client.game_id == body.game_id)
        .for_each(|(_, client)| {
            if let Some(sender) = &client.sender {
                let _ = sender.send(Ok(Message::text(body.message.clone())));
            }
        });

    let client = redis::Client::open(
        env::var("REDIS_HOST").unwrap_or_else(|_e| "redis://127.0.0.1:6379/0".to_string()),
    )
    .unwrap();

    let mut con: redis::aio::Connection = get_con(client).await.unwrap();

    let msg: pubsub::Message = pubsub::Message::new(pubsub::Payload {
        fen: body.message,
        game_id: body.game_id,
    });

    let result = pubsub::publish_message(&mut con, msg).await;

    match result {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => Ok(StatusCode::BAD_REQUEST),
    }
}

pub async fn register_handler(body: RegisterRequest, clients: Clients) -> Result<impl Reply> {
    let game_id: usize = body.game_id;
    let user_id = body.user_id;

    let uuid = Uuid::new_v4().as_simple().to_string();

    register_client(uuid.clone(), user_id, game_id, clients).await;
    Ok(json(&RegisterResponse {
        url: format!("/ws/{}", uuid),
    }))
}

async fn register_client(id: String, user_id: usize, game_id: usize, clients: Clients) {
    clients.write().await.insert(
        id,
        Client {
            user_id,
            game_id,
            sender: None,
        },
    );
}

pub async fn unregister_handler(id: String, clients: Clients) -> Result<impl Reply> {
    clients.write().await.remove(&id);
    Ok(StatusCode::OK)
}

pub async fn ws_handler(ws: warp::ws::Ws, id: String, clients: Clients) -> Result<impl Reply> {
    let client = clients.read().await.get(&id).cloned();
    match client {
        Some(c) => Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, id, clients, c))),
        None => Err(warp::reject::not_found()),
    }
}

pub async fn health_handler() -> Result<impl Reply> {
    Ok(StatusCode::OK)
}
