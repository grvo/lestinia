use std::time::Duration;
use log::info;
use common::clock::Clock;

use client::{
	Input,
	Client,
	Event
};

const FPS: u64 = 60;

fn main() {
	// logging inicial
	pretty_env_logger::init();

	info!("inicializando chat-cli...");

	// configurar um clock de fps
	let mut clock = Clock::new();

	// criar client
	let mut client = Client::new(([127, 0, 0, 1], 59003))
		.expect("falha ao criar uma instância client");

	loop {
		let events = match client.tick(Input::default(), clock.get_last_delta()) {
			Ok(events) => events,

			Err(err) => {
				println!("erro: {:?}", err);

				break;
			}
		};

		for event in events {
			match event {
				Event::Chat(msg) => println!("[chat] {}", msg)
			}
		}

		// limpar o servidor depois de tick
		client.cleanup();

		// esperar pelo próximo tick
		clock.tick(Duration::from_millis(1000 / FPS));
	}
}
