//! IPC via Rabbit MQ

use std::{
    default::Default,
    sync::mpsc::{
        channel,
        Receiver,
    },
};

use super::Error;
use amqp::{
    Basic,
    Options,
    protocol::basic::BasicProperties,
    Session,
    Table,
};
use serde;
use bincode;

pub fn send<T>(queue: &str, msg: &T) -> Result<(), Error>
where T: serde::Serialize {
    let mut s = Session::new( Options {
        .. Default::default()
    })?;
    let mut c = s.open_channel(1)?;
    let _ = c.queue_declare(queue, false, true, false, true, false, Table::new())?;
    let p: BasicProperties = Default::default();
    let msg = bincode::serialize(msg)?;
    c.basic_publish("", queue, true, false, p, msg)?;
    c.close(200, "")?;
    s.close(200, "");
    Ok(())
}

type Rec<T> = Receiver<Result<T, Error>>;
pub fn listen<T: 'static>(queue: &'static str) -> Result<Rec<T>, Error>
where for<'a> T: serde::Deserialize<'a> + Send {
    let (tx, rx) = channel::<Result<T, Error>>();
    let mut s = Session::new(Default::default())?;
    let mut c = s.open_channel(1)?;
    c.basic_prefetch(10)?;
    let _ = c.queue_declare(queue, false, true, false, true, false, Table::new())?;
    ::std::thread::spawn(move || {
        let tx = tx;
        c.basic_consume(move |_: &mut _, _, _, data: Vec<u8>| {
            match bincode::deserialize(&data) {
                Ok(v) => tx.send(Ok(v)).expect("Error sending message to listener"),
                Err(e) => tx.send(Err(e.into())).expect("Error sending de error to listener"),
            }
        }, queue, "", true, true, false, false, Table::new()).expect("Unable to spawn consumer");
        c.start_consuming();
    });
    Ok(rx)
}