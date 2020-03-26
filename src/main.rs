use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

/// Define http actor
struct ChatSession;

impl Actor for ChatSession {
    type Context = ws::WebsocketContext<Self>;
}

type ResultChatMessage = Result<ws::Message, ws::ProtocolError>;

/// Handler for ws::Message message
impl StreamHandler<ResultChatMessage> for ChatSession {
    fn handle(&mut self, msg: ResultChatMessage, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(ChatSession {}, &req, stream);
    println!("{:?}", resp);
    resp
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/websocket/", web::get().to(index)))
        .bind("127.0.0.1:8088")?
        .run()
        .await
}
