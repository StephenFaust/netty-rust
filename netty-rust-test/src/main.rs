use std::{io::Error, thread, time::Duration};

use netty_rust_core::{client::Client, handler::NetHandler, io::chanel::Chanel, server::Server};
use tokio::signal;

pub struct DefaultNetHandler;
pub struct ServerNetHandler;
impl NetHandler for DefaultNetHandler {
    fn on_message(&self, data: Vec<u8>, _: Chanel) {
        let str = String::from_utf8(data).unwrap();
        println!("on_message: {}", str);
    }
    fn on_open(&self) {
        println!("on_open");
    }
    fn on_close(&self) {
        println!("on_close");
    }
    fn on_error(&self, _: Error) {
        println!("on_error");
    }
}

impl NetHandler for ServerNetHandler {
    fn on_message(&self, data: Vec<u8>, chanel: Chanel) {
        let str = String::from_utf8(data).unwrap();
        println!("server_on_message: {}", str);
        chanel
            .write_and_flush("this is server message".as_bytes().to_vec())
            .unwrap()
    }
    fn on_open(&self) {
        println!("on_open");
    }
    fn on_close(&self) {
        println!("on_close");
    }
    fn on_error(&self, err: Error) {
        println!("on_error:{}", err);
    }
}

#[tokio::main]
async fn main() {
    // server
    tokio::spawn(async {
        let rt_boss = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(1)
            .build()
            .unwrap();
        let rt_worker = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(num_cpus::get() << 1)
            .build()
            .unwrap();
        let mut server = Server::new("192.168.30.72:8081", ServerNetHandler, rt_boss, rt_worker);
        server.start().await;
    });

    thread::sleep(Duration::from_secs(2));

    //client
    tokio::spawn(async {
        let rt_worker = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .max_io_events_per_tick(1024)
            .worker_threads(1)
            .build()
            .unwrap();
        let mut client = Client::new("192.168.30.72:8081", DefaultNetHandler, rt_worker);
        let chanel = client.connect().await.unwrap();
        loop {
            thread::sleep(Duration::from_secs(2));
            chanel
                .write_and_flush_async("hello  111".as_bytes().to_vec())
                .await
                .unwrap();
        }
    });

    let ctrl_c = signal::ctrl_c();
    tokio::pin!(ctrl_c);
    tokio::select! {
            _ = ctrl_c => {
                std::process::exit(0);
            }
    }
}
