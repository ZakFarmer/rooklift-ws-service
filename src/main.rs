use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use warp::{ws::Message, Filter, Rejection};

mod handler;
mod pubsub;
mod redis;
mod ws;

type Result<T> = std::result::Result<T, Rejection>;

// Define a type for the clients hashmap (using Arc and RwLock for thread safety)
type Clients = Arc<RwLock<HashMap<String, Client>>>;

#[derive(Debug, Clone)]
pub struct Client {
    pub user_id: usize,
    pub game_id: usize,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

#[tokio::main]
async fn main() {
    // Set CORS config
    let cors = warp::cors()
        .allow_headers(["content-type"])
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "DELETE", "PUT", "OPTIONS"]);

    // Get port from environment at runtime
    // or default to 8080
    let port = std::env::var("PORT")
        .ok()
        .map(|val| val.parse::<u16>())
        .unwrap_or(Ok(8080))
        .unwrap();

    // Initialise empty hashmap for clients
    let clients: Clients = Arc::new(RwLock::new(HashMap::new()));

    // Route for healthcheck (/health)
    let health_route = warp::path!("health").and_then(handler::health_handler);

    // Routes for registering clients
    let register = warp::path("register");
    let register_routes = register
        .and(warp::post())
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(handler::register_handler)
        .or(register
            .and(warp::delete())
            .and(warp::path::param())
            .and(with_clients(clients.clone()))
            .and_then(handler::unregister_handler));

    // Route for broadcasting messages to clients
    let broadcast = warp::path!("broadcast")
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(handler::broadcast_handler);

    // Route for websocket connections
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::path::param())
        .and(with_clients(clients.clone()))
        .and_then(handler::ws_handler);

    // Combine all routes before serving with CORS rules
    let routes = health_route
        .or(register_routes)
        .or(ws_route)
        .or(broadcast)
        .with(cors);

    println!("[Rooklift WS] Server started on port {}", port);

    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}
