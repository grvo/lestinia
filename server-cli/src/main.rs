use std::time::Duration;

use log::info;

use server::{
    Input,
    Event,
    Server
};

use common::clock::Clock;

// tps
const TPS: u64 = 20;

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
                Event::ClientConnected { uid } => info!("cliente {} conectado!", uid),
                Event::ClientDisconnected { uid } => info!("cliente {} desconectado!", uid),

                Event::Chat { uid, msg } => info!("[cliente {}] {}", uid, msg)
            }
        }

        // limpar o servidor depois de tick
        server.cleanup();

        // esperar pelo próximo tick
        clock.tick(Duration::from_millis(1000 / TPS));
    }
}
