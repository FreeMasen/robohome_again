extern crate chrono;
extern crate env_logger;
#[macro_use]
extern crate log;
extern crate robohome_crypto;
extern crate robohome_shared;
extern crate serde_json;
extern crate uuid;
extern crate warp;

use chrono::{Duration, Utc};
use robohome_crypto::{
    bufify_string, gen_pair as gen_auth_key_pair, gen_shared_secret,
};
use robohome_shared::{
    data::{
        get_all_switches, get_auth_age, get_private_shared, new_token,
        update_flip as db_update_flip, update_switch as db_update_switch, Flip, ScheduledFlip,
        Switch,
    },
    ipc::send,
    Error,
};
use serde_json::to_string;
use std::str::FromStr;
use uuid::Uuid;

use warp::{
    body::{concat, content_length_limit, json},
    get2, header,
    http::Response,
    path, post2, put2, Buf, Filter, Reply,
};

fn main() {
    ::std::env::set_var("RUST_LOG", "info");
    env_logger::init();
    let auth_head = header("Authorization");
    let flipping = post2()
        .and(path("flip"))
        .and(auth_head)
        .and(json())
        .map(flip_switch);
    let all_switches = get2()
        .and(path("switches"))
        .and(auth_head)
        .map(get_switches);
    let switch_flips = put2()
        .and(path("switch"))
        .and(auth_head)
        .and(json())
        .map(get_switch_flips);
    let key_exchange = post2()
        .and(path("key-exchange"))
        .and(auth_head)
        .and(content_length_limit(64))
        .and(concat())
        .map(check_auth_token);
    let update_switch = post2()
        .and(path("update_switch"))
        .and(auth_head)
        .and(json())
        .map(update_switch);
    let update_flip = post2()
        .and(path("update_flip"))
        .and(auth_head)
        .and(json())
        .map(update_flip);
    let routes = flipping
        .or(switch_flips)
        .or(all_switches)
        .or(update_switch)
        .or(update_flip)
        .or(key_exchange)
        .or(warp::filters::fs::dir("public"));
    warp::serve(routes.with(warp::log("robohome_flipper"))).run(([0, 0, 0, 0], 3434));
}

fn check_auth_header(header: String) -> Result<bool, Error> {
    let (id, public) = break_auth_header(&header)?;
    let (private, shared) = get_private_shared(&id)?;
    let computed_shared =
        gen_shared_secret(&private, &public).ok_or(Error::new("Invalid public key provided"))?;
    Ok(&*computed_shared == shared.as_slice())
}

fn break_auth_header(header: &str) -> Result<(Uuid, Vec<u8>), Error> {
    let prefix = "Bearer ";
    if !header.starts_with(prefix) {
        return Err(Error::new("Invalid value in Auth header"));
    }
    let header = &header[prefix.len()..];
    let mut split = header.split('$');
    let id_str = split
        .next()
        .ok_or(Error::new("Invalid value in Auth header"))?;
    let public = split
        .next()
        .ok_or(Error::new("Invalid value in Auth header"))?;
    let id = Uuid::from_str(id_str)?;
    let public = bufify_string(public)?;
    Ok((id, public))
}

fn flip_switch(header: String, flip: Flip) -> impl Reply {
    info!("POST /flip: {:?}", flip);
    match check_auth_header(header) {
        Ok(success) => {
            if !success {
                return Response::builder()
                    .status(403)
                    .body(format!(r#"{{"message": "Unauthorized"}}"#));
            }
        }
        Err(e) => {
            let (status, body) = error_response(&e);
            return Response::builder().status(status).body(body);
        }
    }
    match send("switches", &flip) {
        Ok(_) => Response::builder().body(format!(r#"{{"flipped": {}}}"#, flip.code)),
        Err(e) => {
            let (status, body) = error_response(&e);
            Response::builder().status(status).body(body)
        }
    }
}

fn get_switch_flips(header: String, switch: Switch) -> impl Reply {
    info!("PUT /switch");
    match check_auth_header(header) {
        Ok(success) => {
            if !success {
                return Response::builder()
                    .status(403)
                    .body(format!(r#"{{"message": "Unauthorized"}}"#));
            }
        }
        Err(e) => {
            let (status, body) = error_response(&e);
            return Response::builder().status(status).body(body);
        }
    }
    let (status, body) = get_switch_flips_response(switch);
    Response::builder().status(status).body(body)
}

fn get_switches(header: String) -> impl Reply {
    info!("GET /switches");
    match check_auth_header(header) {
        Ok(success) => {
            if !success {
                return Response::builder()
                    .status(403)
                    .body(format!(r#"{{"message": "Unauthorized"}}"#));
            }
        }
        Err(e) => {
            let (status, body) = error_response(&e);
            return Response::builder().status(status).body(body);
        }
    }
    let (status, body) = get_switches_response();
    Response::builder().status(status).body(body)
}

fn update_switch(header: String, switch: Switch) -> impl Reply {
    info!("POST /switch {:?}", switch);
    match check_auth_header(header) {
        Ok(success) => {
            if !success {
                return Response::builder()
                    .status(403)
                    .body(format!(r#"{{"message": "Unauthorized"}}"#));
            }
        }
        Err(e) => {
            let (status, body) = error_response(&e);
            return Response::builder().status(status).body(body);
        }
    }
    let (status, body) = get_update_switch_response(switch);
    Response::builder().status(status).body(body)
}

fn update_flip(header: String, flip: ScheduledFlip) -> impl Reply {
    info!("POST /flip {:?}", flip);
    match check_auth_header(header) {
        Ok(success) => {
            if !success {
                return Response::builder()
                    .status(401)
                    .body(format!(r#"{{"message": "Unauthorized"}}"#));
            }
        }
        Err(e) => {
            let (status, body) = error_response(&e);
            return Response::builder().status(status).body(body);
        }
    }
    let (status, body) = get_update_flip_response(flip);
    Response::builder().status(status).body(body)
}

fn get_update_flip_response(flip: ScheduledFlip) -> (u16, String) {
    match db_update_flip(
        flip.id,
        flip.hour,
        flip.minute,
        flip.dow,
        flip.direction,
        flip.kind,
    ) {
        Ok(flip) => match to_string(&flip) {
            Ok(body) => (200, body),
            Err(e) => error_response(&Error::from(e)),
        },
        Err(e) => error_response(&e),
    }
}

fn get_update_switch_response(switch: Switch) -> (u16, String) {
    match db_update_switch(switch.id, &switch.name, switch.on_code, switch.off_code) {
        Ok(sw) => match to_string(&sw) {
            Ok(body) => (200, body),
            Err(e) => error_response(&Error::from(e)),
        },
        Err(e) => error_response(&e),
    }
}

fn get_switches_response() -> (u16, String) {
    match get_all_switches() {
        Ok(switches) => match to_string(&switches) {
            Ok(body) => (200, body),
            Err(e) => error_response(&Error::from(e)),
        },
        Err(e) => error_response(&e),
    }
}

fn get_switch_flips_response(switch: Switch) -> (u16, String) {
    match robohome_shared::data::get_flips_for_switch(switch.id) {
        Ok(flips) => match to_string(&flips) {
            Ok(body) => (200, body),
            Err(e) => error_response(&Error::from(e)),
        },
        Err(e) => error_response(&e),
    }
}

fn check_auth_token(header: String, body: warp::body::FullBody) -> impl Reply {
    info!("POST /key-exchange");
    let (status, body) = match check_auth_token_response(&header, body) {
        Ok((status, body)) => (status, body),
        Err(e) => error_response(&e),
    };
    Response::builder().status(status).body(body)
}

fn check_auth_token_response(
    header: &String,
    body: warp::body::FullBody,
) -> Result<(u16, String), Error> {
    info!("check_auth_token_response {}", header);
    let start = "Bearer ".len();
    let token = &header[start..];
    let token = Uuid::from_str(token)?;
    info!("parsed token");
    let age = get_auth_age(&token)?;
    info!("got auth age");
    if Utc::now().signed_duration_since(age) > Duration::days(1) {
        return Ok((401, format!(r#"{{"status": "expired"}}"#)));
    }
    info!("Extracting body key");
    let buf = body.bytes();
    info!("Generating server pair");
    let key_pair = gen_auth_key_pair();
    info!("getting server private key");
    let my_private = key_pair.sized_priv();
    info!("generating shared key");
    let shared = gen_shared_secret(&my_private, buf)
        .ok_or(Error::new("Failed to generate shared secret"))?;
    info!("storing key info");
    new_token(&my_private, &shared, token)?;
    Ok((200, format!(r#"{{"status": "success"}}"#)))
}

fn error_response(e: &Error) -> (u16, String) {
    (500, format!(r#"{{ "message": "{}" }}"#, e))
}
