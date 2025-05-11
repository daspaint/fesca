
use rand::Rng;
use crate::types::{SecretShare_single_bit};


pub fn generate_shares(secretV:bool) -> (SecretShare_single_bit, SecretShare_single_bit, SecretShare_single_bit) {
    let mut rng = rand::thread_rng();

    let x1 = rng.gen::<bool>();
    let x2 = rng.gen::<bool>();
    let x3 = x1 ^ x2;


    let p1 = SecretShare_single_bit {
        x: x1,
        a: x3 ^ secretV,
    };

    let p2 = SecretShare_single_bit {
        x: x2,
        a: x1 ^ secretV,
    };
    let p3 = SecretShare_single_bit {
        x: x3,
        a: x2 ^ secretV,
    };


    println!("share1: {:?}", p1);
    println!("share2: {:?}", p2); 
    println!("share3: {:?}", p3);
    (p1,p2,p3)
}