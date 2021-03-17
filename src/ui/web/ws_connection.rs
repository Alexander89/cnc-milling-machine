use super::system::System;
use crate::web::types::{Connect, Disconnect, WsCommands, WsCommandsFrom, WsMessages};
use actix::{
    fut, Actor, ActorContext, Addr, AsyncContext, ContextFutureSpawner, Handler,
    Running, StreamHandler,
};
use actix_web_actors::ws;
use std::time::{Duration, Instant};
use uuid::Uuid;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

type WsContext = ws::WebsocketContext<WsConnection>;

/// Define WS actor
#[derive(Debug, Clone)]
pub struct WsConnection {
    id: Uuid,
    system_addr: Addr<System>,
    hb: Instant,
}

impl Actor for WsConnection {
    type Context = WsContext;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        let addr = ctx.address();
        self.system_addr
            .send(Connect {
                addr: addr.recipient(),
                self_id: self.id,
            })
            .into_actor(self)
            .then(|res, _, ctx| {
                match res {
                    Ok(_res) => (),
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.system_addr.do_send(Disconnect { id: self.id });
        Running::Stop
    }
}

impl WsConnection {
    pub fn new(system: Addr<System>) -> WsConnection {
        WsConnection {
            id: uuid::Uuid::new_v4(),
            hb: Instant::now(),
            system_addr: system,
        }
    }

    fn hb(&self, ctx: &mut WsContext) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Disconnecting failed heartbeat");
                act.system_addr.do_send(Disconnect { id: act.id });
                ctx.stop();
                return;
            }

            ctx.ping(b"ping");
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConnection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                ctx.stop();
            }
            Ok(ws::Message::Nop) => (),
            Ok(ws::Message::Text(s)) => {
                if let Ok(cmd) = serde_json::from_str::<WsCommands>(&s) {
                    self.system_addr.do_send(WsCommandsFrom(self.id, cmd));
                } else {
                    println!("got unknown message {}", s);
                }
            }
            Err(e) => panic!(e),
        }
    }
}

impl Handler<WsMessages> for WsConnection {
    type Result = ();

    fn handle(&mut self, msg: WsMessages, ctx: &mut Self::Context) {
        match serde_json::to_string(&msg) {
            Ok(msg) => ctx.text(&msg),
            Err(e) => println!("failed to parse network-package {:?}", e),
        };
    }
}
