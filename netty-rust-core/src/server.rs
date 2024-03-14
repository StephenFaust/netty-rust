use std::{io::{Error, ErrorKind}, sync::Arc};

use tokio::{net::TcpListener, runtime::Runtime, select, sync::mpsc};

use crate::{handler::NetHandler, io::{chanel::Chanel, codec}};



enum HandleType {
    OnMessage,
    OnConnect,
    OnClose,
    OnError,
}

pub struct Server<'a> {
    addr: &'a str,
    rt_boss: Arc<Runtime>,
    rt_worker: Arc<Runtime>,
    handler_rx: mpsc::Receiver<Message>,
    handler_tx: mpsc::Sender<Message>,
    handler: Box<dyn NetHandler + Send>,
}

struct Message {
    data: Option<Vec<u8>>,
    handle_type: HandleType,
    chanel: Option<Chanel>,
    err: Option<Error>,
}

impl Message {
    fn new(data: Option<Vec<u8>>, handle_type: HandleType, chanel: Option<Chanel>,err : Option<Error>) -> Message {
        Message {
            data,
            handle_type,
            chanel,
            err
        }
    }
}

impl<'a> Server<'a> {
    pub fn new(addr: &'a str, handler: impl NetHandler + Send + 'static,rt_boss : Runtime,rt_worker : Runtime) -> Server<'a> {
        let (tx, rx) = mpsc::channel(1024);
        Server {
            addr,
            rt_boss: Arc::new(rt_boss),
            rt_worker: Arc::new(rt_worker),
            handler_rx: rx,
            handler_tx: tx,
            handler: Box::new(handler),
        }
    }

    async fn handle_message(&mut self) {
        while let Some(msg) = self.handler_rx.recv().await {
            match msg.handle_type {
                HandleType::OnConnect => self.handler.on_open(),
                HandleType::OnClose => self.handler.on_close(),
                HandleType::OnError => self.handler.on_error(msg.err.unwrap()),
                HandleType::OnMessage => self
                    .handler
                    .on_message(msg.data.unwrap(), msg.chanel.unwrap()),
            }
        }
    }

    pub async fn start(&mut self) {
        let listener = TcpListener::bind(self.addr).await.unwrap();
        println!("Server listening on {}", self.addr);
        let rt_worker = self.rt_worker.clone();
        let rt_boss = self.rt_boss.clone();
        let handler_tx = self.handler_tx.clone();
        rt_boss.spawn(async move {
            loop {
                let (mut socket, _) = listener.accept().await.unwrap();
                handler_tx
                    .send(Message::new(None, HandleType::OnConnect, None,None))
                    .await
                    .unwrap();
                let handler_tx2 = handler_tx.clone();
                rt_worker.spawn(async move {
                    let (mut reader, mut writer) = socket.split();
                    let ( chtx, mut chrx) = mpsc::channel(1024);
                    loop {
                        select! {
                             data = codec::DefaultCodec::decode(&mut reader)=>{
                                match data {
                                    Ok(data) => {
                                        handler_tx2
                                            .send(Message::new(Some(data), HandleType::OnMessage,Some(Chanel::new(chtx.clone())),None))
                                            .await
                                            .unwrap();
                                    }
                                    Err(err) => {
                                        match err.kind() {
                                            ErrorKind::UnexpectedEof => {
                                                handler_tx2
                                                .send(Message::new(None, HandleType::OnClose,None,Some(err)))
                                                .await
                                                .unwrap();
                                                break;
                                            }
                                            _=>{
                                                handler_tx2
                                                .send(Message::new(None, HandleType::OnError,None,Some(err)))
                                                .await
                                                .unwrap();
                                            break;
                                            }
                                        }
                                      
                                    }
                                }
                             }
                             send_info = chrx.recv() => {
                                if let Some(send_info) = send_info {
                                    codec::DefaultCodec::encode(&mut writer,&send_info).await;
                                }
                            },
                        }
                    }
                });
            }
        });
        self.handle_message().await;
    }
}
