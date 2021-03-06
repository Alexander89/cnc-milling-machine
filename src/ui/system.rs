use crate::ui::types::{
    Connect, Disconnect, WsCommandsFrom, WsConnectedMessage, WsControllerMessage, WsMessages,
    WsPositionMessage, WsStatusMessage,
};
use actix::{Actor, Addr, Context, Handler, Recipient};
use crossbeam_channel::{Receiver, Sender};
use std::collections::HashMap;
use std::thread;
use uuid::Uuid;

type Connection = Recipient<WsMessages>;
type WsReceiver = Receiver<WsMessages>;
type WsSender = Sender<WsCommandsFrom>;

#[derive(Debug, Clone)]
pub struct SystemState {
    position: WsPositionMessage,
    status: WsStatusMessage,
    controller: WsControllerMessage,
}

/// Define System actor
#[derive(Debug, Clone)]
pub struct System {
    sessions: HashMap<Uuid, Connection>,
    sender: WsSender,
    receiver: WsReceiver,
    last_state: SystemState,
}

type SystemCtx = Context<System>;

impl Actor for System {
    type Context = SystemCtx;
}

impl System {
    pub fn new(
        sender: WsSender,
        receiver: WsReceiver,
        position: WsPositionMessage,
        status: WsStatusMessage,
        controller: WsControllerMessage,
    ) -> Addr<System> {
        let sys = System {
            sessions: HashMap::<Uuid, Connection>::new(),
            sender,
            receiver: receiver.clone(),
            last_state: SystemState {
                position,
                status,
                controller,
            },
        };

        let sys_actor = sys.start();
        let m_sys_actor = sys_actor.clone();
        thread::spawn(move || 'threadLoop: loop {
            match receiver.recv() {
                Ok(msg) => {
                    m_sys_actor.do_send(msg);
                }
                _ => break 'threadLoop,
            }
        });

        sys_actor
    }
    fn send_message(&self, msg: WsMessages) {
        self.sessions
            .iter()
            .for_each(|(id, _)| self.send_message_to(msg.clone(), id))
    }
    fn send_message_to(&self, msg: WsMessages, id_to: &Uuid) {
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            socket_recipient.do_send(msg).unwrap();
        } else {
            println!("attempting to send message but couldn't find user id.");
        }
    }
}

impl Handler<Connect> for System {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut SystemCtx) -> Self::Result {
        println!("add message {}", msg.self_id);
        self.sessions.insert(msg.self_id, msg.addr);
        let welcome = WsMessages::Connected(WsConnectedMessage {
            id: msg.self_id.to_string(),
        });
        self.send_message_to(welcome, &msg.self_id);
        self.send_message_to(
            WsMessages::Position(self.last_state.position.clone()),
            &msg.self_id,
        );
        self.send_message_to(
            WsMessages::Controller(self.last_state.controller.clone()),
            &msg.self_id,
        );
        self.send_message_to(
            WsMessages::Status(self.last_state.status.clone()),
            &msg.self_id,
        );
    }
}

impl Handler<Disconnect> for System {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut SystemCtx) -> Self::Result {
        if self.sessions.remove(&msg.id).is_some() {
            println!("client {} is gone", msg.id);
        }
    }
}

impl Handler<WsMessages> for System {
    type Result = ();

    fn handle(&mut self, msg: WsMessages, _: &mut SystemCtx) -> Self::Result {
        if let WsMessages::Reply { to, msg } = msg {
            let new_msg = WsMessages::Reply { to, msg };
            self.send_message_to(new_msg, &to);
        } else {
            match msg {
                WsMessages::Controller(ref controller) => {
                    self.last_state.controller = controller.clone()
                }
                WsMessages::Position(ref position) => self.last_state.position = position.clone(),
                WsMessages::Status(ref status) => self.last_state.status = status.clone(),
                _ => (),
            }
            self.send_message(msg);
        }
    }
}

impl Handler<WsCommandsFrom> for System {
    type Result = ();

    fn handle(&mut self, msg: WsCommandsFrom, _: &mut SystemCtx) -> Self::Result {
        self.sender.send(msg).unwrap();
    }
}
