use std::net::SocketAddr;
use std::sync::Arc;

use api::docker_crud;
use api::projects::executor::ProjectIoExecutor;
use api::projects::project_store::ProjectStore;
use axum::routing::{delete, post};
use axum::{http::StatusCode, routing::get, Router};

use bollard::Docker;
use store::Store;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::config::{IO_LOG_FOLDER, STORE_FILE, STORE_LOCATION};

pub mod api;
pub mod config;
pub mod store;
pub mod util;

#[derive(Debug, Clone)]
pub struct AppState {
    pub docker: Arc<Mutex<Docker>>,
    pub projects: ProjectStore,
    pub io_executor: Arc<ProjectIoExecutor>,
}

pub type SharedAppState = AppState;

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

    let io_executor = Arc::new(ProjectIoExecutor::new(16));

    let state: SharedAppState = AppState {
        docker: Arc::new(Mutex::new(docker)),
        projects: ProjectStore::new(Store::new_str(STORE_LOCATION, STORE_FILE).unwrap()).await,
        io_executor,
    };

    let volumes_router = Router::new()
        .route("/", get(docker_crud::volume::volumes))
        .route("/:name/remove", delete(docker_crud::volume::remove_volume))
        .route("/prune", delete(docker_crud::volume::prune_volumes));
    let containers_router = Router::new()
        .route("/", get(docker_crud::container::containers))
        .route(
            "/:name/remove",
            delete(docker_crud::container::remove_container),
        )
        .route("/:name/stop", post(docker_crud::container::stop_container))
        .route(
            "/:name/start",
            post(docker_crud::container::start_container),
        )
        .route(
            "/:name/restart",
            post(docker_crud::container::restart_container),
        )
        .route("/:id/logs", get(api::docker_log_ws::ws_upgrader));
    let images_router = Router::new()
        .route("/", get(docker_crud::image::images))
        .route("/:name/remove", delete(docker_crud::image::remove_images))
        .route("/prune", delete(docker_crud::image::prune_images));
    let networks_router = Router::new()
        .route("/", get(docker_crud::network::networks))
        .route(
            "/:name/remove",
            delete(docker_crud::network::remove_network),
        )
        .route("/prune", delete(docker_crud::network::prune_networks));

    let projects_router = Router::new()
        .route("/", get(api::projects::routes::list_projects_route))
        .route("/", post(api::projects::routes::new_project_route))
        .route("/", delete(api::projects::routes::remove_project_route))
        .route("/pull", get(api::projects::routes::pull_project_route))
        .route(
            "/webhook/:name/:branch",
            post(api::projects::routes::webhook_route),
        )
        .route(
            "/action/:name/:branch",
            post(api::projects::routes::project_action_route),
        )
        .route(
            "/io/:id/:kind",
            get(api::projects::iohandle_ws::ws_upgrader),
        )
        .route(
            "/io/current",
            get(api::projects::routes::list_current_builds),
        )
        .nest(
            "/io/history",
            Router::new()
                .route("/", get(api::projects::routes::list_builds))
                .nest_service("/files", ServeDir::new(IO_LOG_FOLDER)),
        );

    // Static files
    let serve_dir = ServeDir::new("public/servcur/build")
        .not_found_service(ServeFile::new("public/servcur/build/index.html"));

    let static_file_router = Router::new()
        .nest_service("/assets", serve_dir.clone())
        .fallback_service(serve_dir);

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/system", get(docker_crud::docker_sys_info))
        .nest("/volumes", volumes_router)
        .nest("/containers", containers_router)
        .nest("/images", images_router)
        .nest("/networks", networks_router)
        .nest("/projects", projects_router)
        .nest("/app", static_file_router)
        .with_state(state)
        .layer(
            CorsLayer::new()
                // allow requests from any origin
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http());
    // run our app with hyper
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
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
