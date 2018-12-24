extern crate chrono;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate robohome_shared;

use std::{
    sync::mpsc::{
        channel,
        Receiver,
    },
    thread::{
        Builder,
        sleep,
    },
    time::Duration,
};

use chrono::{
    Utc,
    Timelike,
};

use robohome_shared::{
    data::{
        Flip,
        get_flips_for_today,
    },
    Error,
    ipc::{
        listen,
        send
    },
};

fn main() -> Result<(), Error> {
    ::std::env::set_var("RUST_LOG", "robohome");
    env_logger::init();
    let (tx, rx) = channel();
    let timer_tx = tx.clone();

    let _ = Builder::new()
            .name(format!("timer_thread"))
            .spawn(move || {
                let tx = timer_tx;
                loop {
                    let _ = tx.send(Message::Tick);
                    sleep(Duration::from_secs(60));
                }
            });
    let (lookup_tx, lookup_rx) = channel();
    let tx2 = tx.clone();
    let _ = Builder::new()
        .name(format!("lookup_thread"))
        .spawn(move || {
            let out = tx2;
            let rx = lookup_rx;
            let mut all_day: Vec<Flip> = get_flips_for_today().unwrap_or(vec![]);
            debug!(target: "robohome", "initial flips {:#?}", all_day);
            loop {
                match rx.recv() {
                    Ok(msg) => match msg {
                        Message::Tick => {
                            let now = Utc::now();
                            let for_sending = all_day.clone().into_iter().filter(|f| {
                                f.hour == (now.time().hour() as i32) && f.minute == (now.time().minute() as i32)
                            }).collect();
                            let _ = out.send(Message::Flips(for_sending));
                        },
                        Message::Refresh => {
                            match get_flips_for_today() {
                                Ok(today) => all_day = today,
                                Err(e) => {
                                    error!("Failed to get flips for today: {}", e);
                                    all_day = Vec::new();
                                }
                            }
                        },
                        _ => (),
                    },
                    Err(e) => error!(target: "robohome", "lookup_thread error: {}", e),
                }
            }
        });
    let _ = Builder::new()
        .name(format!("ipc_thread"))
        .spawn(move || {
            let tx = tx;
            let rx: Receiver<Result<(), Error>> = match listen("database") {
                Ok(rx) => rx,
                Err(e) => {
                    eprintln!("Failed to create mq listener: {}", e);
                    ::std::process::exit(1);
                }
            };
            loop {
                match rx.recv() {
                    Ok(_) => {
                        let _ = tx.send(Message::Refresh);
                    },
                    Err(e) => error!(target: "robohome", "ipc_thread error: {}", e),
                }
            }
        });
    loop {
        match rx.recv() {
            Ok(msg) => {
                debug!(target: "robohome", "Main Thread: {:?}", msg);
                match msg {
                    Message::Tick => {
                        lookup_tx.send(Message::Tick).expect("Failed to send tick message");
                    },
                    Message::Flips(flips) => {
                        for ref flip in flips {
                            send("switches", &flip).expect("Failed to send flip message");
                        }
                    },
                    Message::Refresh => {
                        lookup_tx.send(Message::Refresh).expect("Failed to send refresh message");
                    }
                }
            },
            Err(e) => return Err(e.into()),
        }
    }
}
#[derive(Debug)]
enum Message {
    Flips(Vec<Flip>),
    Refresh,
    Tick,
}