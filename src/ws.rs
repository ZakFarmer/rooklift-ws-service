use std::{collections::HashMap, sync::Arc};

use crate::Client;
use futures::{FutureExt, StreamExt};
use serde::Deserialize;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

// Define a type for the clients hashmap (using Arc and Mutex for thread safety)
pub type Clients = Arc<Mutex<HashMap<String, Client>>>;

pub async fn client_connection(ws: WebSocket, id: String, clients: Clients, mut client: Client) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            eprintln!("Error sending WebSocket msg: {}", e);
        }
    }));

    client.sender = Some(client_sender);
    clients.lock().await.insert(id.clone(), client);

    println!("{} connected", id);

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Error receiving WS message for id: {}): {}", id.clone(), e);
                break;
            }
        };
        client_msg(&id, msg, &clients).await;
    }

    clients.lock().await.remove(&id);
    println!("{} disconnected", id);
}

async fn client_msg(id: &str, msg: Message, clients: &Clients) {
    println!("Received message from {}: {:?}", id, msg);
    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };

    if message == "ping" || message == "ping\n" {
        let locked = clients.lock().await;

        match locked.get(id) {
            Some(v) => {
                if let Some(sender) = &v.sender {
                    println!("Sent \"pong\" to client {}", id);
                    let _ = sender.send(Ok(Message::text("pong")));
                }
            }
            None => return,
        }

        return;
    }
}
