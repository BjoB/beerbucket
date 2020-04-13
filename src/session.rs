use std::time::{Duration, Instant};

use actix::*;
use actix_web_actors::ws;

use serde::{Deserialize, Serialize};
use serde_json;

use crate::server::{self, ChatServer};
/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WsChatSession {
    /// unique session id
    id: usize,
    /// heartbeat
    hb: Instant,
    /// joined room
    room: String,
    /// peer name
    name: Option<String>,
    /// chat server
    addr: Addr<ChatServer>,
}

#[derive(Serialize, Deserialize)]
struct IncomingMessage {
    /// request name for function
    /// TODO: Enum Type like in Frontend
    request: u8,
    /// content, like parameters
    content: Option<String>,
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start.
    /// We register ws session with ChatServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // we'll start heartbeat process on session start.
        self.hb(ctx);

        // register self in chat server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsChatSessionState, state is shared
        // across all routes within application
        let addr = ctx.address();
        self.addr
            .send(server::Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // notify chat server
        self.addr.do_send(server::Disconnect { id: self.id });
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<server::Message> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: server::Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        println!("WEBSOCKET MESSAGE: {:?}", msg);
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                // TODO: check if conversion worked
                let message: IncomingMessage = serde_json::from_str(&text).unwrap();
                match message.request {
                    //message
                    0 => {
                        let msg = if let Some(ref name) = self.name {
                            format!("{}: {}", name, message.content.unwrap())
                        } else {
                            message.content.unwrap().to_owned()
                        };
                        // send message to chat server
                        self.addr.do_send(server::ClientMessage {
                            id: self.id,
                            msg,
                            room: self.room.clone(),
                        })
                    }
                    //list
                    1 => self.list_available_rooms(ctx),
                    //name
                    2 => {
                        if message.content.is_some() {
                            self.name = message.content;
                        } else {
                            ctx.text("!!! name is required");
                        }
                    }
                    //join
                    3 => {
                        if message.content.is_some() {
                            self.join_room(&message.content.unwrap(), ctx);
                        } else {
                            ctx.text("Room name is required!");
                        }
                    }
                    _ => println!("Maybe Error Handling, but it should never run in this case"),
                }
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(_) => {
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}

impl WsChatSession {
    pub fn new(addr: Addr<ChatServer>, name: Option<String>) -> WsChatSession {
        WsChatSession {
            addr,
            name,
            id: 0,
            hb: Instant::now(),
            room: "Main".to_owned(),
        }
    }
    /// helper method that sends ping to client every second.
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // notify chat server
                act.addr.do_send(server::Disconnect { id: act.id });

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }

    fn list_available_rooms(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        println!("List availabe rooms:");
        self.addr
            .send(server::ListRooms)
            .into_actor(self)
            .then(|res, _, ctx| {
                match res {
                    Ok(rooms) => {
                        for room in rooms {
                            ctx.text(room);
                        }
                    }
                    _ => println!("Something is wrong!"),
                }
                fut::ready(())
            })
            .wait(ctx)
        // .wait(ctx) pauses all events in context,
        // so actor wont receive any new messages until it get list
        // of rooms back
    }

    fn join_room(&mut self, room_name: &str, ctx: &mut ws::WebsocketContext<Self>) {
        match self.name.as_ref() {
            Some(nickname) => {
                self.room = room_name.to_owned();
                self.addr.do_send(server::Join {
                    id: self.id,
                    name: self.room.clone(),
                });
                ctx.text(format!("{} joined!", nickname));
            }
            None => {
                println!("Joining channel not allowed without name!");
            }
        }
    }
}
