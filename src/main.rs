use specs::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::time::Duration;
use tungstenite::server::accept;
use tungstenite::{Message, WebSocket};

struct SocketComponent {
    stream: WebSocket<TcpStream>,
}

impl Component for SocketComponent {
    type Storage = VecStorage<Self>;
}

struct MessageComponent {
    messages: Vec<Message>,
}

impl Component for MessageComponent {
    type Storage = VecStorage<Self>;
}

struct SocketReadSystem;

impl<'a> System<'a> for SocketReadSystem {
    type SystemData = (
        WriteStorage<'a, SocketComponent>,
        WriteStorage<'a, MessageComponent>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut sockets, mut messages) = data;
        for socket in (&mut sockets).join() {
            match socket.stream.read_message() {
                Ok(msg) => {
                    if msg.is_binary() || msg.is_text() {
                        println!("Message Received: {}", &msg);
                        for message in (&mut messages).join() {
                            message.messages.push(msg.clone());
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

struct SocketWriteSystem;

impl<'a> System<'a> for SocketWriteSystem {
    type SystemData = (
        WriteStorage<'a, SocketComponent>,
        WriteStorage<'a, MessageComponent>,
    );

    fn run(&mut self, (mut socket, mut message): Self::SystemData) {
        for (socket, message) in (&mut socket, &mut message).join() {
            for msg in message.messages.iter() {
                socket.stream.write_message(msg.clone()).unwrap();
            }
            message.messages.clear();
        }
    }
}

fn main() {
    let mut world = World::new();

    let mut dispatcher = DispatcherBuilder::new()
        .with(SocketReadSystem, "socket_read_system", &[])
        .with(
            SocketWriteSystem,
            "socket_write_system",
            &["socket_read_system"],
        )
        .build();

    dispatcher.setup(&mut world);

    let server = TcpListener::bind("127.0.0.1:3000").unwrap();

    server.set_nonblocking(true).unwrap();
    loop {
        while let Ok((stream, _)) = server.accept() {
            let websocket = accept(stream).unwrap();
            world
                .create_entity()
                .with(SocketComponent { stream: websocket })
                .with(MessageComponent { messages: vec![] })
                .build();
        }

        dispatcher.dispatch(&world);

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60)); // 60 fps (ish)
    }
}
