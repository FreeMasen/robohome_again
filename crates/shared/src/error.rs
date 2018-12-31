use std::{
    error::Error as StdError,
    fmt::{
        Formatter,
        Result as FmtRes,
        Display
    },
    sync::mpsc::{
        RecvError
    },
    num::ParseIntError as IError,
};

use serde::{
    Serialize,
    Serializer,
    ser::{
        SerializeMap
    },
};

use serde_json::Error as JsonError;

use amqp::AMQPError;
use bincode;
use postgres::Error as PError;
use uuid::ParseError as UError;

#[derive(Debug)]
pub enum Error {
    Bin(bincode::Error),
    Json(JsonError),
    Mq(AMQPError),
    Other(String),
    Pg(PError),
    Recv(RecvError),
    Uuid(UError),
    U8(IError),
}

impl Error {
    pub fn new(msg: &str) -> Self {
        Error::Other(msg.to_owned())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtRes {
        match self {
            Error::Bin(e) => e.fmt(f),
            Error::Json(e) => e.fmt(f),
            Error::Mq(e) => e.fmt(f),
            Error::Other(msg) => msg.fmt(f),
            Error::Pg(e) => e.fmt(f),
            Error::Recv(e) => e.fmt(f),
            Error::Uuid(e) => e.fmt(f),
            Error::U8(e) => e.fmt(f),
        }
    }
}

impl StdError for Error {
    fn cause(&self) -> Option<&StdError> {
        match self {
            Error::Bin(ref e) => Some(e),
            Error::Json(ref e) => Some(e),
            Error::Mq(ref e) => Some(e),
            Error::Other(_) => None,
            Error::Pg(ref e) => Some(e),
            Error::Recv(ref e) => Some(e),
            Error::Uuid(ref e) => Some(e),
            Error::U8(ref e) => Some(e),
        }
    }
}

impl From<AMQPError> for Error {
    fn from(other: AMQPError) -> Self {
        Error::Mq(other)
    }
}

impl From<bincode::Error> for Error {
    fn from(other: bincode::Error) -> Self {
        Error::Bin(other)
    }
}

impl From<PError> for Error {
    fn from(other: PError) -> Self {
        Error::Pg(other)
    }
}

impl From<RecvError> for Error {
    fn from(other: RecvError) -> Self {
        Error::Recv(other)
    }
}

impl From<JsonError> for Error {
    fn from(other: JsonError) -> Self {
        Error::Json(other)
    }
}

impl From<UError> for Error {
    fn from(other: UError) -> Self {
        Error::Uuid(other)
    }
}

impl From<IError> for Error {
    fn from(other: IError) -> Self {
        Error::U8(other)
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry("message", &format!("{}", self))?;
        map.end()
    }
}