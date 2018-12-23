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
            Ok(f) => {
                if let Err(e) = send(f.code, 17, 180) {
                    eprintln!("Failed to send: {}\n{}", f.code, e);
                } else {
                    eprintln!("Successfully sent {}", f.code);
                }
            },
            Err(e) => panic!("Error recv: {}", e),
        };
    }
}
