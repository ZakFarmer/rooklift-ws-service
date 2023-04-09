use redis::{aio::Connection, AsyncCommands};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub channel: String,
    pub payload: Payload,
}

impl Message {
    pub fn new(payload: Payload) -> Message {
        Self {
            id: Message::generate_id(),
            channel: String::from("games"),
            payload,
        }
    }

    fn generate_id() -> String {
        Uuid::new_v4().to_string()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    pub fen: String,
    pub game_id: usize,
}

// Publish a message to a Redis channel
pub async fn publish_message(
    con: &mut Connection,
    message: Message,
) -> Result<(), Box<dyn std::error::Error>> {
    let json: String = serde_json::to_string(&message)?;

    println!("Publishing message \"{}\" to channel \"{}\"", json, message.channel);

    con.publish(message.channel, json).await?;
    Ok(())
}