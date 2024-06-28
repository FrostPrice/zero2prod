use actix_web::{dev::Server, web, App, HttpServer};
use std::net::TcpListener;

use super::routes::{health_check::health_check, subscriptions::subscribe};

// This is no longer a binary entrypoint. Now you can use it as a library in other binaries or tests.
pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("subscriptions", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();
    // No await here!
    // Let others do the awaiting for this server.
    Ok(server)
}
