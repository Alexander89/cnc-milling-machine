pub mod types;

use crate::ui::types::{WsCommands, WsMessages};
use actix::{Actor, StreamHandler};
use actix_cors::Cors;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use crossbeam_channel::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use crossbeam_channel::unbounded;

type WsReceiver = Receiver<WsMessages>;
type WsSender = Sender<WsCommands>;

/// Define HTTP actor
#[derive(Debug, Clone)]
struct MyWs {
    receiver: WsReceiver,
    sender: WsSender,
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            e => println!("other {:?}", e),
        }
    }
}

async fn index(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<Mutex<MyWs>>
) -> Result<HttpResponse, Error> {
    println!("data {:?}", data);
    let resp = ws::start(data.lock().unwrap().clone(), &req, stream);
    println!("send resp {:?}", resp);
    resp
}

#[actix_web::main]
pub async fn ui_main(sender: WsSender, receiver: WsReceiver) -> std::io::Result<()> {
    println!("ws started");

    HttpServer::new(move || {
        let data = web::Data::new(Mutex::new(MyWs {
            sender: sender.clone(),
            receiver: receiver.clone(),
        }));
        App::new()
            .wrap(Cors::permissive())
            .app_data(data.clone())
            .route("/ws", web::get().to(index))
    })
    .bind("127.0.0.1:1506")?
    .run()
    .await
}
