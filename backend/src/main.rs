use std::{
    env,
    io::Error,
    iter,
    ops::Deref,
    sync::Arc,
    time::{Duration, Instant},
};

use futures_util::StreamExt;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::RwLock,
};

mod place;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    let shared_state: Arc<RwLock<place::State>> = Arc::new(RwLock::new(place::State::new()));
    {
        let mut state_guard = shared_state.write().await;
        let state = &mut *state_guard;
        state.set_grid_size(1000, 1000).await;
        let now = Instant::now();
        let cloned_grid = state.read_grid().await;
        println!("{}", now.elapsed().as_millis());
    }

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream, shared_state.clone()));
    }

    Ok(())
}

async fn accept_connection(stream: TcpStream, _state: Arc<RwLock<place::State>>) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    println!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    println!("New WebSocket connection: {}", addr);

    let (_write, _read) = ws_stream.split();
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    loop {
        interval.tick().await;

        // println!("{:?}", state.read());
        // let response = write.send(Message::Text(state.to_string())).await;
        // match response {
        //     Ok(_) => {}
        //     Err(_) => break,
        // }
    }

    // We should not forward messages other than text or binary.
    // read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
    //     .forward(write)
    //     .await
    //     .expect("Failed to forward messages")
}
