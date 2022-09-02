use std::{
    env,
    io::Error,
    iter,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use futures_util::{SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::RwLock,
};
use tokio_tungstenite::tungstenite::Message;

type Pixel = RwLock<(u8, u8, u8)>;

type Row = RwLock<Vec<Pixel>>;

type Grid = RwLock<Vec<Row>>;

struct State {
    grid: Grid,
}
impl State {
    fn new() -> Self {
        State {
            grid: RwLock::new(Vec::new()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    let shared_state: Arc<State> = Arc::new(State::new());
    {
        let now = Instant::now();
        let mut guard = shared_state.grid.write().await;
        guard.append(
            &mut iter::repeat_with(|| {
                RwLock::new(
                    iter::repeat_with(|| RwLock::new((255, 255, 255)))
                        .take(1000)
                        .collect(),
                )
            })
            .take(1000)
            .collect(),
        );
        println!("{}", now.elapsed().as_millis());
    }

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream, shared_state.clone()));
    }

    Ok(())
}

async fn accept_connection(stream: TcpStream, state: Arc<State>) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    println!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    println!("New WebSocket connection: {}", addr);

    let (mut write, _read) = ws_stream.split();
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
