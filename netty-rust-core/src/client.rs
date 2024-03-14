use std::{io::ErrorKind, sync::Arc};

use tokio::{
    net::TcpStream,
    runtime::Runtime,
    select,
    sync::mpsc::{self, Receiver, Sender},
};

use crate::{
    handler::NetHandler,
    io::{chanel::Chanel, codec},
};

pub struct Client<'a> {
    pub addr: &'a str,
    recver: Option<Receiver<Vec<u8>>>,
    sender: Sender<Vec<u8>>,
    rt_worker: Arc<Runtime>,
    handler: Option<Box<dyn NetHandler + Send>>,
}
impl<'a> Client<'a> {
    pub fn new(
        addr: &'a str,
        handler: impl NetHandler + Send + 'static,
        rt_worker: Runtime,
    ) -> Client<'a> {
        let (tx, rx) = mpsc::channel(1024);
        Client {
            addr,
            recver: Some(rx),
            sender: tx,
            rt_worker: Arc::new(rt_worker),
            handler: Some(Box::new(handler)),
        }
    }

    pub async fn connect(&mut self) -> Result<Chanel, String> {
        let mut socket: TcpStream = TcpStream::connect(self.addr).await.unwrap();
        let mut rx = self.recver.take().unwrap();
        let handler = self.handler.take().unwrap();
        let tx = self.sender.clone();
        let tx2 = tx.clone();
        self.rt_worker.spawn(async move {
            let (mut reader, mut writer) = socket.split();
            loop {
                select! {
                    send_info = rx.recv() => {
                        if let Some(send_info) = send_info {
                            codec::DefaultCodec::encode(&mut writer,&send_info).await;
                        }
                    },
                    data = codec::DefaultCodec::decode(&mut reader) => {
                     match data {
                        Ok(data) => {
                            handler.on_message(data,Chanel::new(tx.clone()));
                        },
                        Err(err) => {

                            match err.kind() {
                                ErrorKind::UnexpectedEof => {
                                    handler.on_close();
                                    break;
                                }
                                _=>{
                                    handler.on_error(err);
                                break;
                                }
                            }
                    }
                }
                     },
                }
            }
        });
        Ok(Chanel::new(tx2.clone()))
    }
}
