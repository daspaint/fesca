
use rand::Rng;

pub fn generate_coorelated_single_bit() -> (bool, bool, bool) {
    let mut rng = rand::thread_rng();

    let alpha = rng.gen::<bool>();
    let beta = rng.gen::<bool>();
    let gamma = alpha ^ beta;

    (alpha, beta, gamma)
}