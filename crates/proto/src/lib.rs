use futures_util::{SinkExt, StreamExt};
use ring::aead::{Aad, CHACHA20_POLY1305, LessSafeKey, NONCE_LEN, Nonce, UnboundKey};
use std::io;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub mod backend;
pub mod frontend;

pub struct DashboardSocket {
    framed: Framed<TcpStream, LengthDelimitedCodec>,
    key: LessSafeKey,
}

impl DashboardSocket {
    pub fn new(stream: TcpStream, key: [u8; 32]) -> Self {
        let framed = LengthDelimitedCodec::builder()
            .length_field_type::<u32>()
            .new_framed(stream);

        // Length of buffer is guaranteed to be 32 bytes
        let key = UnboundKey::new(&CHACHA20_POLY1305, &key).unwrap();
        let key = LessSafeKey::new(key);

        Self { framed, key }
    }

    pub async fn read_frame<F: bitcode::DecodeOwned>(&mut self) -> Result<Option<F>, io::Error> {
        self.framed
            .next()
            .await
            .map(|x| {
                x.and_then(|mut data| {
                    let nonce = data.split_off(data.len() - NONCE_LEN);
                    let nonce = Nonce::assume_unique_for_key((*nonce).try_into().unwrap());

                    let data = self
                        .key
                        .open_in_place(nonce, Aad::empty(), &mut data)
                        .map_err(|_| io::Error::other("decryption failed"))?;

                    bitcode::decode(data)
                        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err.to_string()))
                })
            })
            .transpose()
    }

    pub async fn write_frame<F: bitcode::Encode>(&mut self, frame: F) -> Result<(), io::Error> {
        let mut data = bitcode::encode(&frame);

        // Random nonces start to become unsafe after about 2^30 messages sent with the same key
        let nonce: [u8; 12] = rand::random();

        self.key
            .seal_in_place_append_tag(Nonce::assume_unique_for_key(nonce), Aad::empty(), &mut data)
            .map_err(|_| io::Error::other("encryption failed"))?;

        data.extend(nonce);

        self.framed.send(data.into()).await
    }
}
