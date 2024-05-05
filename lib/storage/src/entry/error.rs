#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid magic")]
    InvalidMagic,

    #[error("prost encode")]
    ProstEncode(#[from] prost::EncodeError),

    #[error("prost decode")]
    ProstDecode(#[from] prost::DecodeError),
}
