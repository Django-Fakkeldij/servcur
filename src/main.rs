use std::sync::Arc;

use axum::{extract::State, http::StatusCode, routing::get, Router};

use bollard::{container::ListContainersOptions, Docker};
use tokio::sync::Mutex;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone)]
struct AppState {
    state: Arc<Mutex<Docker>>,
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "servcur=trace,tower_http=trace,axum::rejection=trace".into()),
        )
        .init();

    // Init docker api connection
    let docker = Docker::connect_with_socket_defaults().unwrap();

    let state: AppState = AppState {
        state: Arc::new(Mutex::new(docker)),
    };

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/containers", get(containers))
        .with_state(state)
        .layer(TraceLayer::new_for_http());
    // run our app with hyper
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> (StatusCode, &'static str) {
    (StatusCode::OK, "hi")
}

async fn containers(State(state): State<AppState>) -> (StatusCode, String) {
    let ret = state
        .state
        .lock_owned()
        .await
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await
        .unwrap();
    (StatusCode::OK, serde_json::to_string_pretty(&ret).unwrap())
}
