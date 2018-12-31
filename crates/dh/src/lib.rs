extern crate x25519_dalek;
extern crate rand;
extern crate wasm_bindgen;
extern crate robohome_shared;

use robohome_shared::Error;

use wasm_bindgen::prelude::*;

use x25519_dalek::{
    generate_secret,
    generate_public,
    diffie_hellman,
};

use rand::thread_rng;

#[wasm_bindgen]
pub struct KeyPair {
    public: Box<[u8]>,
    private: Box<[u8]>,
}

#[wasm_bindgen]
impl KeyPair {
    pub fn public(&self) -> Box<[u8]> {
        self.public.clone()
    }

    pub fn private(&self) -> Box<[u8]> {
        self.private.clone()
    }
}

impl KeyPair {
    pub fn sized_pub(&self) -> Vec<u8> {
        let mut ret = Vec::with_capacity(32);
        for i in 0..32 {
            ret[i] = self.public[i];
        }
        ret
    }

    pub fn sized_priv(&self) -> Vec<u8> {
        let mut ret = Vec::with_capacity(32);
        for i in 0..32 {
            ret.push(self.private[i]);
        }
        ret
    }
}

#[wasm_bindgen]
pub fn gen_pair() -> KeyPair {
    let mut rng = thread_rng();
    let private: [u8; 32] = generate_secret(&mut rng);
    let public: [u8; 32] = generate_public(&private).to_bytes();
    KeyPair {
        public: Box::new(public),
        private: Box::new(private),
    }
}
#[wasm_bindgen]
pub fn gen_shared_secret(my_private: &[u8], their_public: &[u8]) -> Option<Box<[u8]>> {
    if my_private.len() != 32 || their_public.len() != 32 {
        return None;
    }
    let mut sized_private: [u8; 32] = [0; 32];
    let mut sized_public: [u8; 32] = [0; 32];
    for i in 0..32 {
        sized_private[i] = my_private[i];
        sized_public[i] = their_public[i];
    }
    Some(Box::new(diffie_hellman(&sized_private, &sized_public)))
}

pub fn stringify_buf(buf: &[u8]) -> String {
    buf.iter().map(|i| format!("{:02x}", i)).collect()
}

pub fn bufify_string(s: &str) -> Result<Vec<u8>, Error> {
    let mut idx = 0;
    let mut ret = Vec::with_capacity(32);
    while idx < 64 {
        let pair = &s[idx..idx + 2];
        let n = u8::from_str_radix(pair, 16)?;
        ret.push(n);
        idx += 2;
    }
    Ok(ret)
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn full() {
        let pair1 = gen_pair();
        let pair2 = gen_pair();
        let pub1 = pair1.public();
        let priv1 = pair1.private();
        let pub2 = pair2.public();
        let priv2 = pair2.private();
        let shared1 = gen_shared_secret(&*priv1, &*pub2);
        let shared2 = gen_shared_secret(&*priv2, &*pub1);
        assert_eq!(shared1, shared2);
    }

    #[test]
    fn round_trip() {
        for _ in 0..100 {
            let pair1 = gen_pair();
            let one = pair1.private();
            let two = pair1.public();
            let one_str = stringify_buf(&one);
            let two_str = stringify_buf(&two);
            let one_back = bufify_string(&one_str).unwrap();
            let two_back = bufify_string(&two_str).unwrap();
            assert_eq!(&*one, one_back.as_slice());
            assert_eq!(&*two, two_back.as_slice());
        }
    }
}