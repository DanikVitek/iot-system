use std::{sync::Arc, thread};

use actix_web::{
    middleware::{NormalizePath, TrailingSlash},
    web, App, HttpServer,
};
use color_eyre::eyre::Result;
use iot_system::{config::TryRead, proto, setup_tracing, KtConvenience};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    config::Configuration,
    control::{grpc, ws::Subscribers},
};

mod config;
mod control;
mod data;
mod error;
mod service;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let _guard = setup_tracing("./logs", "lab2.log")?;

    let config = Configuration::try_read()?;
    tracing::debug!("Configuration: {:#?}", config);

    let pool: PgPool = PgPool::connect_with(config.database().connect_options()).await?;
    tracing::info!("Connected to database");

    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Migrations successfully applied");

    let subs = Arc::new(Subscribers::new());

    let store_service = grpc::StoreService::new(subs.clone(), pool.clone());
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()?;

    let openapi = ApiDocs::openapi();

    tokio::spawn({
        let address = config.grpc_server().into();
        async move {
            tonic::transport::Server::builder()
                .add_service(reflection_service)
                .add_service(proto::store_server::StoreServer::new(store_service))
                .serve(address)
                .await?;
            Ok::<(), tonic::transport::Error>(())
        }
    });

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(
                web::scope("/api")
                    .wrap(NormalizePath::new(TrailingSlash::Trim))
                    .service(control::ws::ws_endpoint)
                    .service(control::http::create_processed_agent_data)
                    .service(control::http::read_processed_agent_data)
                    .service(control::http::read_processed_agent_data_list)
                    .service(control::http::update_processed_agent_data)
                    .service(control::http::delete_processed_agent_data)
                    .app_data(web::Data::new(pool.clone()))
                    .app_data(web::Data::from(subs.clone())),
            )
            .service(web::redirect("/swagger-ui", "/swagger-ui/"))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            .also(|_| tracing::info!("App built for worker {:?}", thread::current().id()))
    })
    .bind(config.http_server())?
    .run()
    .await?;

    Ok(())
}

#[derive(OpenApi)]
#[openapi(
    paths(
        control::http::create_processed_agent_data,
        control::http::read_processed_agent_data,
        control::http::read_processed_agent_data_list,
        control::http::update_processed_agent_data,
        control::http::delete_processed_agent_data,
    ),
    components(
        schemas(
            data::Accelerometer,
            data::Gps,
            data::Agent,
            data::ProcessedAgent,
            data::ProcessedAgentWithId
        ),
        responses(
            data::Accelerometer,
            data::Gps,
            data::Agent,
            data::ProcessedAgent,
            data::ProcessedAgentWithId
        ),
    )
)]
struct ApiDocs;
