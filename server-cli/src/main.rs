// padrão
use std::time::Duration;

// biblioteca
use log::info;

// projeto
use server::{
    self,

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
    let mut server = Server::new();

    loop {
        server.tick(server::Input {}, clock.get_last_delta())
            .expect("falha ao tickar o servidor");

        // limpar o servidor depois de tick
        server.cleanup();

        // esperar pelo próximo tick
        clock.tick(Duration::from_millis(1000 / FPS));
    }
}
