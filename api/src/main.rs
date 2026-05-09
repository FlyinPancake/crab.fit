use std::{env, net::SocketAddr, sync::Arc};

use axum::{
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    serve, Extension,
};

use tokio::net::TcpListener;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::adaptors::{create_adaptor, AnyAdaptor};
use crate::docs::ApiDoc;

mod adaptors;
mod docs;
mod errors;
mod payloads;
mod routes;

pub type Router = utoipa_axum::router::OpenApiRouter<()>;
pub type AdaptorExtension = Extension<Arc<AnyAdaptor>>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // Load env
    dotenvy::dotenv().ok();

    let adaptor = create_adaptor().await;

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_methods([Method::GET, Method::POST, Method::PATCH])
        .allow_origin(
            if cfg!(debug_assertions) {
                "http://localhost:1234".to_owned()
            } else {
                env::var("FRONTEND_URL").expect("Missing FRONTEND_URL environment variable")
            }
            .parse::<HeaderValue>()
            .unwrap(),
        );

    // Rate limiting configuration (using tower_governor)
    // From the docs: Allows bursts with up to 20 requests and replenishes
    // one element after 500ms, based on peer IP.
    let governor_config = Arc::new(
        GovernorConfigBuilder::default()
            .burst_size(20)
            .finish()
            .unwrap(),
    );
    let rate_limit = GovernorLayer::new(governor_config);

    let (app, openapi) = routes::router().split_for_parts();

    let app = app
        .merge(
            SwaggerUi::new("/docs")
                .url("/docs/openapi.json", ApiDoc::openapi().merge_from(openapi)),
        )
        .layer(cors)
        .layer(Extension(Arc::new(adaptor)))
        .layer(rate_limit)
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    tracing::info!(
        "Crab Fit API listening at http://{} in {} mode",
        addr,
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );

    let listener = TcpListener::bind(addr).await.unwrap();
    serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler")
    })
    .await
    .unwrap();
}
