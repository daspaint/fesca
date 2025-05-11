use serde::{Serialize, Deserialize};



#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretShare_single_bit {
    pub x: bool,
    pub a: bool,
}



// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct SecretShare {
//     pub x: Vec<u8>,
//     pub a: Vec<u8>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub enum Operation {
//     And,
//     Xor,
// }


// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct TripleShare {
//     pub alpha: Vec<u8>, // Correlated randomness for AND protocol
//     pub beta: Vec<u8>,
//     pub gamma: Vec<u8>,
// }

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct NetworkMessage {
//     pub client_id: usize,
//     pub share: SecretShare,
// }