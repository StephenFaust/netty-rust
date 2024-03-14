use std::io::Error;

use crate::io::chanel::Chanel;

pub trait NetHandler {
    fn on_message(&self, data: Vec<u8>, chanel: Chanel);
    fn on_open(&self);
    fn on_close(&self);
    fn on_error(&self, err: Error);
}
