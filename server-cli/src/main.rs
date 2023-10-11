use std::time::Duration;

use log::info;

use server::{
    Input,
    Event,
    Server
};

use common::clock::Clock;

// fps
const FPS: u64 = 60;

fn main() {
    // logging inicial
    pretty_env_logger::init();

    info!("inicializando server-cli...");

    // configurar clock de fps
    let mut clock = Clock::new();

    // criar servidor
    let mut server = Server::new()
        .expect("falha ao criar instância de servidor");

    loop {
        let events = server.tick(Input::default(), clock.get_last_delta())
            .expect("falha ao tickar o servidor");

        for event in events {
            match event {
                Event::ClientConnected { ecs_entity } => println!("cliente conectado!"),
                Event::ClientDisconnected { ecs_entity } => println!("cliente desconectado!"),

                Event::Chat { msg, .. } => println!("[chat] {}", msg)
            }
        }

        // limpar o servidor depois de tick
        server.cleanup();

        // esperar pelo próximo tick
        clock.tick(Duration::from_millis(1000 / FPS));
    }
}
