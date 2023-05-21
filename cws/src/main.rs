use axum::{response::IntoResponse};
use futures::{StreamExt, SinkExt};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};


#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("cws=info"))
        .with(tracing_subscriber::fmt::layer())
        .init();
    let app = router();
    let addr = ([127, 0, 0, 1], 3001).into();
    tracing::info!("Lauching Server...");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}


struct AppState {
    pub users: std::sync::Mutex<std::collections::HashSet<String>>,
    tx: tokio::sync::broadcast::Sender<String>,
}



fn router() -> axum::Router {

    let (tx, _) = tokio::sync::broadcast::channel(100);
    let users = std::sync::Mutex::new(std::collections::HashSet::new());
    let state = std::sync::Arc::new(AppState { users, tx });

    tracing::info!("WS :: Listening on :: 127.0.0.1:3000");
    axum::Router::new()
        .route("/", axum::routing::get(index))
        .route("/ws", axum::routing::get(ws))
        .with_state(state)
}



async fn index() -> impl IntoResponse {
    axum::response::Html(std::include_str!("../../chat.html"))
}


async fn ws(
    ws: axum::extract::ws::WebSocketUpgrade, 
    axum::extract::State(state): axum::extract::State<std::sync::Arc<AppState>>
) -> impl IntoResponse {
    ws.on_upgrade(|so| _ws(so, state))
}

async fn _ws(
    so: axum::extract::ws::WebSocket,
    state: std::sync::Arc<AppState>,
) {
    let (mut tx, mut rx) = so.split();
    let mut name = String::new();
    while let Some(Ok(msg)) = rx.next().await {
        if let axum::extract::ws::Message::Text(m) = msg {
            match exists_name(&state, &m) {
                Ok(nm) => {
                    name = nm.to_owned();
                    break;
                },
                Err(err) => {
                    if let ExistsError::UsernameTaken = err {
                        let _ = tx
                            .send(axum::extract::ws::Message::Text("Username already taken.".into()))
                        .await;
                    }
                    return;
                }
            };
        };
    }

    let mut st_rx = state.tx.subscribe();
    let msg = format!("{} joined.", name);
    tracing::info!("Sending :: {}", msg);
    let _ = state.tx.send(msg);

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = st_rx.recv().await {
            if tx.send(axum::extract::ws::Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });



    let st_tx = state.tx.clone();
    let uname = name.clone();

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(axum::extract::ws::Message::Text(text))) = rx.next().await {
            let _ = st_tx.send(format!("{} :: {}", uname, text));
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    let msg = format!("{} left", name);
    tracing::info!("Sending :: {}", msg);
    let _ = state.tx.send(msg);

    state.users.lock().unwrap().remove(&name);
}

fn exists_name<'a>(state: &AppState, nm: &'a str) -> ExistsResult<&'a str> {
    let users = state.users.lock();

    if let Err(_) = users {
        return Err(ExistsError::UsersLock);
    }

    if users.unwrap().contains(nm) {
        return Err(ExistsError::UsernameTaken);
    }

    Ok(nm)
}

type ExistsResult<T> = std::result::Result<T, ExistsError>;

#[derive(Debug, thiserror::Error)]
enum ExistsError {
    #[error("Unable to obtain users set lock.")]
    UsersLock,

    #[error("Username already taken")]
    UsernameTaken,
}
