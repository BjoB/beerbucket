use actix::*;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

mod server;
mod session;
mod messages;

use session::WsChatSession;

/// Entry point for our route
async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::ChatServer>>
) -> Result<HttpResponse, Error> {
    ws::start(
        WsChatSession::new(srv.get_ref().clone(), None),
        &req,
        stream,
    )
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let server = server::ChatServer::default().start();

    // Create Http server with websocket support
    HttpServer::new(move || {
        App::new()
            .data(server.clone())
            .service(web::resource("/ws/").to(chat_route))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
