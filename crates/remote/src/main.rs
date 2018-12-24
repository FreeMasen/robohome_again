extern crate robohome_shared;
extern crate codesender;

use robohome_shared::{
    ipc::listen,
    data::Flip,
};

use codesender::send;

fn main() {
    let rx = listen::<Flip>("switches")
                .expect("Failed to establish listener");
    loop {
        match rx.recv() {
            Ok(r) => handle_message(r),
            Err(e) => panic!("Error recv: {}", e),
        };
    }
}


fn handle_message(r: Result<Flip, Error>) {
    match r {
        Ok(f) send_code(f),
        Err(e) => eprintln!("{}", e);
    }
}

fn send_code(f: Flip) {
    match send(f.code as usize, 17, 180) {
        Ok(_) => println!("Successfully sent {}", f.code),
        Err(e) => eprintln!("Failed to send: {}\n{}", f.code, e),
    }
}