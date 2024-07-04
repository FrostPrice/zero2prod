use super::email_client::EmailClient;
use super::routes::{health_check::health_check, subscriptions::subscribe};
use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

// This is no longer a binary entrypoint. Now you can use it as a library in other binaries or tests.
pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    // Make connection an ARC
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);

    // Capture `connection` in the closure
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("subscriptions", web::post().to(subscribe))
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();
    // No await here!
    // Let others do the awaiting for this server.
    Ok(server)
}
