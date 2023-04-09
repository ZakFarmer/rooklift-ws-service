# WebSocket service #

This is a small web service that will handle the realtime interactions between players.

It's written in Rust, using the [Warp framework](https://github.com/seanmonstar/warp) for the web server and websocket handling and [tokio](https://github.com/tokio-rs/tokio) for the async runtime. 