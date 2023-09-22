#![feature(euclidean_division, duration_float, try_from, trait_alias)]

#[macro_use]
extern crate serde_derive;

pub mod clock;
pub mod comp;
pub mod figure;
pub mod state;
pub mod terrain;
pub mod util;
pub mod volumes;
pub mod vol;

// todo: não ignorar o código aqui, por alguma razão isso refusa para compilar enquanto não tenha nenhum problema de cópia e cola
/// o módulo de networking contém wrappers de alto nível do `tcplistener` e `tcpstream` e dados utilizados por ambos os servidores e cliente
///
/// # exemplos
/// ```ignore
/// use std::net::SocketAddr;
///
/// use lestinia_common::net::{
/// 	PostOffice,
/// 	PostBox
/// }
///
/// let listen_addr = SocketAddr::from(([0, 0, 0, 0], 12345u16));
/// let conn_addr = SocketAddr::from(([127, 0, 0, 1], 12345u16));
///
/// let server: PostOffice<String, String> = PostOffice::new(&listen_addr).unwrap();
/// let client: PostBox<String, String> = PostBox::to_server(&conn_addr).unwrap();
/// std::thread::sleep(std::time::Duration::from_millis(100));
///
/// let scon = server.get_iter().unwrap().next().unwrap().unwrap();
/// std::thread::sleep(std::time::Duration::from_millis(100));
///
/// scon.send(String::from("foo"));
/// client.send(String::from("bar"));
/// std::thread::sleep(std::time::Duration::from_millis(100));
///
/// assert_eq!("foo", client.recv_iter().unwrap().next().unwrap().unwrap());
/// assert_eq!("bar", scon.recv_iter().unwrap().next().unwrap().unwrap());
/// ```

pub mod net;
