extern crate robohome_shared;
extern crate codesender;

use robohome_shared::{
    ipc::listen,
    data::Flip,
    Error,
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
        Ok(f) => send_code(f),
        Err(e) => eprintln!("{}", e),
    }
}
#[inline(always)]
fn send_code(f: Flip) {
    for _ in 0..10 {
        let _  = send(f.code as usize, 17, 178);
    }
}