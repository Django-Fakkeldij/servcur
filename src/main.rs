use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use api::projects::Project;
use axum::routing::post;
use axum::Json;
use axum::{extract::State, http::StatusCode, routing::get, Router};

use bollard::{
    container::ListContainersOptions, image::ListImagesOptions, network::ListNetworksOptions,
    volume::ListVolumesOptions, Docker,
};
use serde_json::{json, Value};
use store::Store;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::config::{STORE_FILE, STORE_LOCATION};

pub mod api;
pub mod config;
pub mod store;
pub mod util;

#[derive(Debug, Clone)]
pub struct AppState {
    pub docker: Docker,
    pub file_store: Store,
    pub projects: HashMap<String, Project>,
}

pub type SharedAppState = Arc<Mutex<AppState>>;

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

    let state: SharedAppState = Arc::new(Mutex::new(AppState {
        docker,
        file_store: Store::new_str(STORE_LOCATION, STORE_FILE).unwrap(),
        projects: Default::default(),
    }));

    let volumes_router = Router::new().route("/", get(volumes));
    let containers_router = Router::new()
        .route("/", get(containers))
        .route("/:id/logs", get(api::ws::ws_upgrader));
    let images_router = Router::new().route("/", get(images));
    let networks_router = Router::new().route("/", get(networks));

    let projects_router = Router::new()
        .route("/", post(api::projects::routes::new_project_route))
        .route("/pull", get(api::projects::routes::pull_project_route))
        .route(
            "/webhook/:name/:branch",
            post(api::projects::routes::webhook_route),
        )
        .route(
            "/action/:name/:branch",
            post(api::projects::routes::project_action_route),
        );

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

async fn docker_sys_info(State(state): State<SharedAppState>) -> (StatusCode, Json<Value>) {
    let ret = state.lock_owned().await.docker.info().await.unwrap();
    (StatusCode::OK, Json(json!(&ret)))
}

async fn containers(State(state): State<SharedAppState>) -> (StatusCode, Json<Value>) {
    let ret = state
        .lock_owned()
        .await
        .docker
        .list_containers(Some(ListContainersOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await
        .unwrap();
    (StatusCode::OK, Json(json!(&ret)))
}

async fn images(State(state): State<SharedAppState>) -> (StatusCode, Json<Value>) {
    let ret = state
        .lock_owned()
        .await
        .docker
        .list_images(Some(ListImagesOptions::<String> {
            all: true,
            ..Default::default()
        }))
        .await
        .unwrap();
    (StatusCode::OK, Json(json!(&ret)))
}

async fn volumes(State(state): State<SharedAppState>) -> (StatusCode, Json<Value>) {
    let ret = state
        .lock_owned()
        .await
        .docker
        .list_volumes(Some(ListVolumesOptions::<String> {
            ..Default::default()
        }))
        .await
        .unwrap();
    (StatusCode::OK, Json(json!(&ret)))
}

async fn networks(State(state): State<SharedAppState>) -> (StatusCode, Json<Value>) {
    let ret = state
        .lock_owned()
        .await
        .docker
        .list_networks(Some(ListNetworksOptions::<String> {
            ..Default::default()
        }))
        .await
        .unwrap();
    (StatusCode::OK, Json(json!(&ret)))
}
