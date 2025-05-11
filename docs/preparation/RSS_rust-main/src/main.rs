mod generate_shares;
mod types;
mod recover_shares;
mod boolean_primitive;
mod generate_correlated_rand;

use boolean_primitive::{xor_gate_single_bit, and_gate_single_bit};
use generate_shares::generate_shares;
use recover_shares::recover_shares_single_bit;
use types::{SecretShare_single_bit};
use crate::generate_correlated_rand::generate_coorelated_single_bit;


fn main() {
    println!("Replicated Secret Sharing Protocol");
    println!("---------------------------------");
    println!("ref: https://eprint.iacr.org/2016/768.pdf");
    let secretV1 = true;
    let secretV2 = false;
    let secretV3 = true;

    let (x1_s, x2_s, x3_s) = generate_shares(secretV1);
    let (y1_s, y2_s, y3_s) = generate_shares(secretV2);
    let (z1_s, z2_s, z3_s) = generate_shares(secretV3);

    //send shares to each parties x1_s y1_s to P1, x2_s y2_s to P2, x3_s y3_s to P3

    let test_r1 = recover_shares_single_bit(&x1_s, &x2_s);
    println!("Recovered secret1: {:?}", test_r1);

    let test_r2 = recover_shares_single_bit(&y1_s, &y2_s);
    println!("Recovered secret2: {:?}", test_r2);

    let test_r3 = recover_shares_single_bit(&z1_s, &z2_s);
    println!("Recovered secret3: {:?}", test_r3);


    //test xor gate
    let xor_p1_share = xor_gate_single_bit(x1_s, y1_s);
    let xor_p2_share = xor_gate_single_bit(x2_s, y2_s);
    let xor_p3_share = xor_gate_single_bit(x3_s, y3_s);

    println!("secret1 xor secret2 = {:?}",secretV1^secretV2);
    println!("test xor result:{:?}",recover_shares_single_bit(&xor_p2_share, &xor_p3_share));


    //test and gate

    let (alpha, beta, gamma) = generate_coorelated_single_bit();


    //send r1 to P2
    let and_p1_share_r1 = and_gate_single_bit(xor_p1_share, z1_s, alpha);

    //send r2 to P3
    let and_p2_share_r2 = and_gate_single_bit(xor_p2_share, z2_s, beta);

    //send r3 to P1
    let and_p3_share_r3 = and_gate_single_bit(xor_p3_share, z3_s, gamma);

    //p1: r1,r3

    let p1_after_and_share = SecretShare_single_bit{
        x: and_p1_share_r1 ^ and_p3_share_r3,
        a: and_p1_share_r1
    };
    //p2: r1,r2

    let p2_after_and_share = SecretShare_single_bit{
        x: and_p2_share_r2 ^ and_p1_share_r1,
        a: and_p2_share_r2
    };

    //p3: r2,r3

    let p3_after_and_share = SecretShare_single_bit{
        x: and_p3_share_r3 ^ and_p2_share_r2,
        a: and_p3_share_r3
    };

    println!("secret1 xor secret2 and secret3 = {:?}",secretV1^secretV2 & secretV3);
    println!("test AND result:{:?}",recover_shares_single_bit(&p1_after_and_share, &p2_after_and_share));














    
}
