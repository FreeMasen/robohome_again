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

    if ::std::env::var("RUST_LOG").is_err() {
        ::std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    info!("Booting scheduler");
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
            info!("Spawning lookup thread");
            let out = tx2;
            let rx = lookup_rx;
            let mut all_day: Vec<Flip> = match get_flips_for_today() {
                Ok(f) => f,
                Err(e) => {
                    error!("failed to get initial flips for today: {}", e);
                    vec![]
                },
            };
            debug!("initial flips {:#?}", all_day);
            loop {
                match rx.recv() {
                    Ok(msg) => match msg {
                        Message::Tick => {
                            info!("lookup: tick");
                            let now = Utc::now();
                            let now = now.time();
                            let for_sending: Vec<Flip> = all_day.clone().into_iter().filter(|f| {
                                chrono::NaiveTime::from_hms(f.hour as u32, f.minute as u32, 0) < now
                            }).collect();
                            all_day.retain(|f| !for_sending.contains(f));
                            let _ = out.send(Message::Flips(for_sending));
                        },
                        Message::Refresh => {
                            info!("lookup: refresh");
                            match get_flips_for_today() {
                                Ok(today) => all_day = today,
                                Err(e) => {
                                    error!("Failed to get flips for today: {}", e);
                                    all_day = Vec::new();
                                }
                            }
                        },
                        _ => info!("Unknown message"),
                    },
                    Err(e) => error!(target: "robohome", "lookup_thread error: {}", e),
                }
            }
        });
    let _ = Builder::new()
        .name(format!("db_update_thread"))
        .spawn(move || {
            info!("spawning db update thread");
            let tx = tx;
            let db_rx: Receiver<Result<(), Error>> = match listen("database") {
                Ok(rx) => rx,
                Err(e) => {
                    error!("Failed to create mq listener for database updates: {}", e);
                    ::std::process::exit(1);
                }
            };
            loop {
                match db_rx.recv() {
                    Ok(_) => {
                        info!("Sending refresh message");
                        let _ = tx.send(Message::Refresh);
                    },
                    Err(e) => error!("ipc_thread error: {}", e),
                }
            }
        });
    loop {
        match rx.recv() {
            Ok(msg) => {
                debug!("Main Thread: {:?}", msg);
                match msg {
                    Message::Tick => {
                        if let Err(e) = lookup_tx.send(Message::Tick){
                            error!("Failed to send tick message {}", e);
                        }
                    },
                    Message::Flips(flips) => {
                        for ref flip in flips {
                            if let Err(e) = send("switches", &flip){
                                error!("Failed to send flip message {}", e);
                            }
                        }
                    },
                    Message::Refresh => {
                        if let Err(e) = lookup_tx.send(Message::Refresh){
                            error!("Failed to send refresh message {}", e);
                        }
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