use specs::prelude::*;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;
use tungstenite::server::accept;

#[derive(Debug)]
struct Vel(f32);

impl Component for Vel {
    type Storage = VecStorage<Self>;
}

#[derive(Debug)]
struct Pos(f32);

impl Component for Pos {
    type Storage = VecStorage<Self>;
}

struct SysA;

impl<'a> System<'a> for SysA {
    type SystemData = (WriteStorage<'a, Pos>, ReadStorage<'a, Vel>);

    fn run(&mut self, (mut pos, vel): Self::SystemData) {
        for (pos, vel) in (&mut pos, &vel).join() {
            pos.0 += vel.0;
            println!("position: {:?}", pos);
        }
    }
}

fn main() {
    let mut world = World::new();

    let mut dispatcher = DispatcherBuilder::new().with(SysA, "sys_a", &[]).build();

    dispatcher.setup(&mut world);

    world.create_entity().with(Vel(2.0)).with(Pos(0.0)).build();
    world.create_entity().with(Vel(4.0)).with(Pos(1.6)).build();
    world.create_entity().with(Vel(1.5)).with(Pos(5.4)).build();

    let server = TcpListener::bind("127.0.0.1:3000").unwrap();

    server.set_nonblocking(true).unwrap();
    loop {
        dispatcher.dispatch(&world);

        // Accept all tcp connections before moving on
        while let Ok((stream, _)) = server.accept() {
            stream.set_nonblocking(false).unwrap();
            thread::spawn(move || {
                let mut websocket = accept(stream).unwrap();
                loop {
                    let msg = websocket.read_message().unwrap();

                    if msg.is_binary() || msg.is_text() {
                        println!("Message: {}", msg);
                        websocket.write_message(msg).unwrap();
                    }
                }
            });
        }

        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 1)); // 1 fps
    }
}
