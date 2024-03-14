use tokio::sync::mpsc::{
    error::{SendError, TrySendError},
    Sender,
};
pub struct Chanel {
    sender: Sender<Vec<u8>>,
}

impl Clone for Chanel {
    fn clone(&self) -> Self {
        Chanel::new(self.sender.clone())
    }
}

impl Chanel {
    pub fn new(sender: Sender<Vec<u8>>) -> Self {
        Self { sender }
    }

    pub async fn write_and_flush_async(&self, data: Vec<u8>) -> Result<(), SendError<Vec<u8>>> {
        self.sender.send(data).await
    }

    pub fn write_and_flush(&self, data: Vec<u8>) -> Result<(), TrySendError<Vec<u8>>> {
        self.sender.try_send(data)
    }
}
