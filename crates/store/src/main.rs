use std::thread;

use actix_web::{
    middleware::{NormalizePath, TrailingSlash},
    web, App, HttpServer,
};
use color_eyre::eyre::Result;
use iot_system::{config::TryRead, setup_tracing, KtConvenience};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{config::Configuration, control::ws::Subscribers};

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

    let openapi = ApiDocs::openapi();

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
                    .app_data(web::Data::new(Subscribers::new())),
            )
            .service(web::redirect("/swagger-ui", "/swagger-ui/"))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
            .also(|_| tracing::info!("App built for worker {:?}", thread::current().id()))
    })
    .bind(config.server())?
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
