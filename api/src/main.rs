use std::{net::SocketAddr, sync::Arc};

use axum::{
    Extension,
    http::{
        HeaderValue, Method,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    },
    serve,
};

use color_eyre::eyre::ContextCompat;
use tokio::net::TcpListener;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::docs::ApiDoc;
use crate::{adaptors::AnyAdaptor, config::Config};

mod adaptors;
mod config;
mod docs;
mod errors;
mod payloads;
mod routes;

pub type Router = utoipa_axum::router::OpenApiRouter<()>;
pub type AdaptorExtension = Extension<Arc<AnyAdaptor>>;
pub type ConfigExtension = Extension<Arc<Config>>;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // Load env
    dotenvy::dotenv().ok();

    let config = Config::load()?;

    let adaptor: AnyAdaptor = match config.adaptor_kind {
        config::AdaptorConfig::Memory => memory_adaptor::MemoryAdaptor::new().await.into(),
        config::AdaptorConfig::Sql => sql_adaptor::SqlAdaptor::new(
            &config
                .database_url
                .clone()
                .context("validation should have caught this")?,
        )
        .await
        .into(),
        config::AdaptorConfig::Datastore => datastore_adaptor::DatastoreAdaptor::new().await.into(),
    };

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_methods([Method::GET, Method::POST, Method::PATCH])
        .allow_origin(config.frontend_url.parse::<HeaderValue>()?);

    // Rate limiting configuration (using tower_governor)
    // From the docs: Allows bursts with up to 20 requests and replenishes
    // one element after 500ms, based on peer IP.
    let governor_config = Arc::new(
        GovernorConfigBuilder::default()
            .burst_size(20)
            .finish()
            .context("failed to set up governor")?,
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
        .layer(Extension(Arc::new(config.clone())))
        .layer(rate_limit)
        .layer(TraceLayer::new_for_http());

    let addr = config.http_listen_addr.parse::<SocketAddr>()?;

    tracing::info!(
        "Crab Fit API listening at http://{} in {} mode",
        addr,
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );

    let listener = TcpListener::bind(addr).await?;
    serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler")
    })
    .await?;

    Ok(())
}
