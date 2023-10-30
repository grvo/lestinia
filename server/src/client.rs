use specs::Entity as EcsEntity;

use common::{
	comp,
	
	msg::{
		ServerMsg,
		ClientMsg
	},

	net::PostBox
};

use crate::Error;

pub struct Client {
	pub uid: comp::Uid,
	pub postbox: PostBox<ServerMsg, ClientMsg>,
	pub last_ping: f64
}

pub struct Clients {
	clients: Vec<Client>
}

impl Clients {
	pub fn empty() -> Self {
		Self {
			clients: Vec::new()
		}
	}

	pub fn add(&mut self, client: Client) {
		self.clients.push(client);
	}

	pub fn remove_if<F: FnMut(&mut Client) -> bool>(&mut self, f: F) {
		self.clients.drain_filter(f);
	}

	pub fn notify_all(&mut self, msg: ServerMsg) {
		for client in &mut self.clients {
			// consumir qualquer erro; acabar com eles
			let _ = client.postbox.send(msg.clone());

			println!("enviando mensagem...");
		}
	}

	pub fn notify_all_except(&mut self, uid: comp::Uid, msg: ServerMsg) {
		for client in &mut self.clients {
			if client.uid != uid {
				// consome qualquer erros, tentando resolvê-los depois
				let _ = client.postbox.send(msg.clone());

				println!("enviando mensagem...");
			}
		}
	}
}
