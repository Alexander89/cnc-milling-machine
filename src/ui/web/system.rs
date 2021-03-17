use crate::web::types::{
    Connect, Disconnect, UiCommandsFrom, UiConnectedMessage, UiControllerMessage, UiMessages,
    UiPositionMessage, UiStatusMessage,
};
use actix::{Actor, Addr, Context, Handler, Recipient};
use crossbeam_channel::{Receiver, Sender};
use std::collections::HashMap;
use std::thread;
use uuid::Uuid;

type Connection = Recipient<UiMessages>;
type WsReceiver = Receiver<UiMessages>;
type WsSender = Sender<UiCommandsFrom>;

#[derive(Debug, Clone)]
pub struct SystemState {
    position: UiPositionMessage,
    status: UiStatusMessage,
    controller: UiControllerMessage,
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
        position: UiPositionMessage,
        status: UiStatusMessage,
        controller: UiControllerMessage,
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
    fn send_message(&self, msg: UiMessages) {
        self.sessions
            .iter()
            .for_each(|(id, _)| self.send_message_to(msg.clone(), id))
    }
    fn send_message_to(&self, msg: UiMessages, id_to: &Uuid) {
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
        let welcome = UiMessages::Connected(UiConnectedMessage {
            id: msg.self_id.to_string(),
        });
        self.send_message_to(welcome, &msg.self_id);
        self.send_message_to(
            UiMessages::Position(self.last_state.position.clone()),
            &msg.self_id,
        );
        self.send_message_to(
            UiMessages::Controller(self.last_state.controller.clone()),
            &msg.self_id,
        );
        self.send_message_to(
            UiMessages::Status(self.last_state.status.clone()),
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

impl Handler<UiMessages> for System {
    type Result = ();

    fn handle(&mut self, msg: UiMessages, _: &mut SystemCtx) -> Self::Result {
        if let UiMessages::Reply { to, msg } = msg {
            let new_msg = UiMessages::Reply { to, msg };
            self.send_message_to(new_msg, &to);
        } else {
            match msg {
                UiMessages::Controller(ref controller) => {
                    self.last_state.controller = controller.clone()
                }
                UiMessages::Position(ref position) => self.last_state.position = position.clone(),
                UiMessages::Status(ref status) => self.last_state.status = status.clone(),
                _ => (),
            }
            self.send_message(msg);
        }
    }
}

impl Handler<UiCommandsFrom> for System {
    type Result = ();

    fn handle(&mut self, msg: UiCommandsFrom, _: &mut SystemCtx) -> Self::Result {
        self.sender.send(msg).unwrap();
    }
}
