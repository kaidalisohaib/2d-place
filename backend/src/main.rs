use std::{
    env,
    io::Error,
    ptr,
    sync::Arc,
    time::{Duration, Instant},
};

use bebop::{prelude::*, SubRecord};
use futures_util::{future, FutureExt, SinkExt, StreamExt, TryStreamExt};
use generated::grid::*;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{
        broadcast::{self, Receiver, Sender},
        mpsc, RwLock,
    },
};
use tokio_tungstenite::tungstenite::Message;

mod generated;
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
        let now = Instant::now();
        state.set_grid_size(2000, 2000);
        state.set_new_encoded_data();
        println!("{:?}", now.elapsed());
    }

    let (sender_grid_manipulator, receiver_grid_manipulator) = mpsc::channel::<Vec<u8>>(16);
    let (sender_clients, receiver_clients) = broadcast::channel::<Vec<u8>>(16);

    tokio::spawn(grid_manipulator(
        receiver_grid_manipulator,
        sender_clients.clone(),
    ));

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(
            stream,
            shared_state.clone(),
            sender_grid_manipulator.clone(),
            sender_clients.subscribe(),
        ));
    }

    Ok(())
}

async fn accept_connection<'a>(
    stream: TcpStream,
    state: Arc<RwLock<place::State>>,
    sender_grid_manipulator: mpsc::Sender<Vec<u8>>,
    mut receiver_clients: broadcast::Receiver<Vec<u8>>,
) {
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    // println!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    let (mut write, read) = ws_stream.split();
    let now = Instant::now();
    let mut response = write
        .send(Message::Binary(
            state.read().await.get_encoded_data_cloned(),
        ))
        .await;

    // If the full grid transaction failed we return
    if let Err(_) = response {
        return;
    }

    let receive_future = read
        .try_filter(|msg| future::ready(msg.is_binary()))
        .try_for_each(|msg| {
            if let Err(_) = sender_grid_manipulator.try_send(msg.into_data()) {
                return future::ok(());
            }
            future::ok(())
        });

    let send_future = async {
        while let Ok(encoded_data) = receiver_clients.recv().await {
            write.send(Message::Binary(encoded_data)).await;
        }
    };

    println!("{:?}", now.elapsed());
    future::join(receive_future, send_future).await;
    println!("finished");
}

async fn grid_manipulator(
    mut receiver_grid_manipulator: mpsc::Receiver<Vec<u8>>,
    sender_clients: broadcast::Sender<Vec<u8>>,
) {
    while let Some(encoded_data) = receiver_grid_manipulator.recv().await {
        let decoded_pixel =
            Pixel::deserialize(&BebopData::deserialize(&encoded_data).unwrap().encoded_data)
                .unwrap();
        println!("{:?}", decoded_pixel);
        sender_clients.send(encoded_data);
    }
}
