use std::io::Error;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpStream,
    },
};
trait Codec {
    fn decode(socket: TcpStream) -> Vec<u8>;
    fn encode(data: &[u8]);
}

pub struct DefaultCodec;

pub struct HttpCodec;

impl<'a> HttpCodec {
    pub async fn decode(read_socket: &mut ReadHalf<'a>) -> Result<Vec<u8>, String> {
        let mut data = Vec::new();
        loop {
            let mut buf = [0; 1024];
            match read_socket.read(&mut buf).await {
                Ok(n) => {
                    if n != 0 {
                        data.extend_from_slice(&buf[0..n]);
                        return Ok(data);
                    }
                }
                Err(e) => {
                    println!("error: {}", e);
                    return Err("error".to_string());
                }
            }
        }
    }
}

impl<'a> DefaultCodec {
    pub async fn decode(read_socket: &mut ReadHalf<'a>) -> Result<Vec<u8>, Error> {
        let mut buf = [0; 8];
        let mut count: usize = 0;
        let mut length: usize = 0;
        match read_socket.read(&mut buf).await {
            Ok(n) => {
                if n == 0 {
                } else {
                    length = usize::from_be_bytes(buf);
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                return Err(e);
            }
        }
        let mut buf_vec: Box<[u8]> = vec![0; length].into_boxed_slice();
        while count < length {
            match read_socket.read(&mut buf_vec[count..length]).await {
                Ok(n) => {
                    count += n;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        return Ok(buf_vec.to_vec());
    }

    pub async fn encode(write_socket: &mut WriteHalf<'a>, data: &Vec<u8>) {
        let length = data.len();
        write_socket.write_u64(length as u64).await.unwrap();
        write_socket.write_all(data).await.unwrap();
        write_socket.flush().await.unwrap();
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        tokio::task::spawn(async {});
    }
}
