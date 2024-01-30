use std::sync::Arc;

use axum::extract::Path;
use axum::Json;
use axum::{extract::State, http::StatusCode, routing::get, Router};

use futures::stream::StreamExt;

use bollard::container::{AttachContainerOptions, LogOutput};
use bollard::{
    container::ListContainersOptions, image::ListImagesOptions, network::ListNetworksOptions,
    volume::ListVolumesOptions, Docker,
};
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
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

    // Ping to make sure docker is running
    docker
        .ping()
        .await
        .expect("Could not connect to Docker daemon (is it running?)");

    let state: AppState = AppState {
        state: Arc::new(Mutex::new(docker)),
    };

    let volumes_router = Router::new().route("/", get(volumes));
    let containers_router = Router::new()
        .route("/", get(containers))
        .route("/:id/logs", get(container_logs));
    let images_router = Router::new().route("/", get(images));
    let networks_router = Router::new().route("/", get(networks));

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/system", get(docker_sys_info))
        .nest("/volumes", volumes_router)
        .nest("/containers", containers_router)
        .nest("/images", images_router)
        .nest("/networks", networks_router)
        .with_state(state)
        .layer(
            CorsLayer::new()
                // allow requests from any origin
                .allow_origin(Any)
                .allow_headers(Any),
        )
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

async fn docker_sys_info(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    let ret = state.state.lock_owned().await.info().await.unwrap();
    (StatusCode::OK, Json(json!(&ret)))
}

async fn containers(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
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
    (StatusCode::OK, Json(json!(&ret)))
}

// TODO: make it WS
// TODO: query param to skip chunks (avoid having lots of frontend traffic)
async fn container_logs(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<Value>) {
    let mut ret = state
        .state
        .lock_owned()
        .await
        .attach_container(
            &id,
            Some(AttachContainerOptions::<String> {
                stdin: Some(true),
                stdout: Some(true),
                stderr: Some(true),
                stream: Some(true),
                logs: Some(true),
                detach_keys: Some("ctrl-c".to_string()),
            }),
        )
        .await
        .unwrap()
        .output;
    let ret1: LogOutput = ret.next().await.unwrap().unwrap();
    let m = match ret1 {
        LogOutput::StdErr { message } => message,
        LogOutput::StdOut { message } => message,
        LogOutput::StdIn { message } => message,
        LogOutput::Console { message } => message,
    };
    (
        StatusCode::OK,
        Json(json!(&String::from_utf8(m.to_vec()).unwrap())),
    )
}

async fn images(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    let ret = state
        .state
        .lock_owned()
        .await
        .list_images(Some(ListImagesOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await
        .unwrap();
    (StatusCode::OK, Json(json!(&ret)))
}

async fn volumes(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    let ret = state
        .state
        .lock_owned()
        .await
        .list_volumes(Some(ListVolumesOptions::<String> {
            ..Default::default()
        }))
        .await
        .unwrap();
    (StatusCode::OK, Json(json!(&ret)))
}

async fn networks(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    let ret = state
        .state
        .lock_owned()
        .await
        .list_networks(Some(ListNetworksOptions::<String> {
            ..Default::default()
        }))
        .await
        .unwrap();
    (StatusCode::OK, Json(json!(&ret)))
}
