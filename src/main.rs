use std::net::SocketAddr;
use std::sync::Arc;

use axum::routing::post;
use axum::Json;
use axum::{extract::State, http::StatusCode, routing::get, Router};

use bollard::{
    container::ListContainersOptions, image::ListImagesOptions, network::ListNetworksOptions,
    volume::ListVolumesOptions, Docker,
};
use serde_json::{json, Value};
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

pub mod api;

#[derive(Debug, Clone)]
pub struct AppState {
    state: Arc<Mutex<Docker>>,
}

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "servcur=trace,tower_http=trace,axum::rejection=trace,bollard=info".into()
        }))
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
        .route("/:id/logs", get(api::ws::ws_upgrader));
    let images_router = Router::new().route("/", get(images));
    let networks_router = Router::new().route("/", get(networks));

    let projects_router = Router::new()
        .route("/", post(api::projects::new_project_route))
        .route("/fetch", get(api::projects::fetch_project_route));

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/system", get(docker_sys_info))
        .nest("/volumes", volumes_router)
        .nest("/containers", containers_router)
        .nest("/images", images_router)
        .nest("/networks", networks_router)
        .nest("/projects", projects_router)
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
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
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
