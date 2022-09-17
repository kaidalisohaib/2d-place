use std::{
    env,
    io::Error,
    sync::Arc,
    time::{Duration, Instant},
};

use bebop::{prelude::*};
use futures_util::{future, pin_mut, SinkExt, StreamExt, TryStreamExt};
use generated::grid::*;

use tokio::{
    net::{TcpListener, TcpStream},
    sync::{
        broadcast::{self},
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
        let now = Instant::now();
        let mut state_guard = shared_state.write().await;
        let state = &mut *state_guard;
        state.set_grid_size(1000, 1000).await;
        state.set_new_encoded_grid_data().await;
        println!("{:?}", now.elapsed());
    }

    let (sender_grid_manipulator, receiver_grid_manipulator) = mpsc::channel::<Vec<u8>>(16);
    let (sender_clients, _receiver_clients) = broadcast::channel::<Vec<u8>>(16);

    tokio::spawn(grid_manipulator(
        shared_state.clone(),
        receiver_grid_manipulator,
        sender_clients.clone(),
    ));

    tokio::spawn(full_grid_saver(shared_state.clone()));

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
    println!("Peer address: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    let (mut write, read) = ws_stream.split();
    write
        .feed(Message::Binary(
            state.read().await.get_encoded_grid_data_cloned().await,
        ))
        .await;
    write
        .feed(Message::Binary(
            state.read().await.get_encoded_delta_data_cloned().await,
        ))
        .await;
    write.flush().await;

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
            let now = Instant::now();
            write.send(Message::Binary(encoded_data)).await;
            println!("{:?}", now.elapsed());
        }
    };

    pin_mut!(receive_future, send_future);
    future::select(receive_future, send_future).await;
}

async fn grid_manipulator(
    state: Arc<RwLock<place::State>>,
    mut receiver_grid_manipulator: mpsc::Receiver<Vec<u8>>,
    sender_clients: broadcast::Sender<Vec<u8>>,
) {
    while let Some(encoded_data) = receiver_grid_manipulator.recv().await {
        let decoded_pixel =
            Pixel::deserialize(&BebopData::deserialize(&encoded_data).unwrap().encoded_data)
                .unwrap();
        let state_guard = state.read().await;
        state_guard.set_pixel(decoded_pixel).await;
        state_guard.add_pixel_to_delta(decoded_pixel).await;
        state_guard.set_new_encoded_delta_data().await;
        sender_clients.send(encoded_data);
    }
}

async fn full_grid_saver(state: Arc<RwLock<place::State>>) {
    let mut tick = tokio::time::interval(Duration::from_millis(5000));
    loop {
        tick.tick().await;
        state.read().await.set_new_encoded_grid_data().await;
    }
}
