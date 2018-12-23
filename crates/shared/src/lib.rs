extern crate amqp;
extern crate bincode;
extern crate chrono;
#[macro_use]
extern crate postgres;
#[macro_use]
extern crate postgres_derive;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate uuid;

mod error;
pub mod ipc;
pub mod data;

pub use error::Error;