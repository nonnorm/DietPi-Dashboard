use futures_util::{SinkExt, StreamExt};
use std::io;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub mod backend;
pub mod frontend;

pub struct DashboardSocket(Framed<TcpStream, LengthDelimitedCodec>);

impl DashboardSocket {
    pub fn new(stream: TcpStream) -> Self {
        let framed = LengthDelimitedCodec::builder()
            .length_field_type::<u16>()
            .new_framed(stream);

        Self(framed)
    }

    pub async fn read_frame<F: bitcode::DecodeOwned>(&mut self) -> Result<Option<F>, io::Error> {
        self.0
            .next()
            .await
            .map(|x| {
                x.and_then(|x| {
                    bitcode::decode(&x)
                        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))
                })
            })
            .transpose()
    }

    pub async fn write_frame<F: bitcode::Encode>(&mut self, frame: F) -> Result<(), io::Error> {
        let data = bitcode::encode(&frame).into();

        self.0.send(data).await
    }
}
