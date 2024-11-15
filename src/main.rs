use std::{net::SocketAddr, sync::Arc};
use currency::api;
use tokio::{sync::Mutex, net::TcpListener};
use serde_json::Value;

#[tokio::main]
async fn main() {
    
	let adr = TcpListener::bind("0.0.0.0:0").await.unwrap();

	let socket_address: SocketAddr = format!("0.0.0.0:{port}", port=adr.local_addr().unwrap().port()).parse().expect("Invalid address");

	drop(adr);

	let cache = Arc::new(Mutex::new(Vec::<Value>::new()));

	let routes = api::routes(cache);

	let server = warp::serve(routes).try_bind(socket_address);

	println!("Serving at: http://localhost:{port}", port = socket_address.port());

	server.await;
}
