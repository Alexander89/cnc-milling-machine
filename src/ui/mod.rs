pub mod system;
pub mod types;
pub mod ws_connection;

use actix::Addr;
use actix_cors::Cors;
use actix_web::{get, web::Data, web::Payload, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use crossbeam_channel::{Receiver, Sender};
use system::System;
use types::{WsCommands, WsMessages, WsPositionMessage, WsStatusMessage};
use ws_connection::WsConnection;

type WsReceiver = Receiver<WsMessages>;
type WsSender = Sender<WsCommands>;

#[get("/ws")]
pub async fn web_socket(
    req: HttpRequest,
    stream: Payload,
    srv: Data<Addr<System>>,
) -> Result<HttpResponse, Error> {
    let ws = WsConnection::new(srv.get_ref().clone());

    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}

#[actix_web::main]
pub async fn ui_main(
    sender: WsSender,
    receiver: WsReceiver,
    position: WsPositionMessage,
    status: WsStatusMessage,
) -> std::io::Result<()> {
    let system = System::new(sender, receiver, position, status);

    HttpServer::new(move || {
        App::new()
            .wrap(Cors::permissive())
            .data(system.clone())
            .service(web_socket)
    })
    .bind("127.0.0.1:1506")?
    .run()
    .await
}
