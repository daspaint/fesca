use std::collections::HashMap;

use crate::helpers::secret_share::{SecretShare, SecretShareSend};

pub struct Node {
    pub saved_shares: HashMap<u64, SecretShare>,
    pub received_shares: HashMap<u64, SecretShareSend>,
    pub calculated_shares: HashMap<u64, SecretShare>,
}
impl Node {
    pub fn new() -> Self {
        Node {
            saved_shares: HashMap::new(),
            received_shares: HashMap::new(),
            calculated_shares: HashMap::new(),
        }
    }

    pub fn add_saved_share(&mut self, share: SecretShare) {
        self.saved_shares.insert(share.id, share);
    }

    pub fn add_received_share(&mut self, share: SecretShareSend) {
        self.received_shares.insert(share.id, share);
    }

    pub fn add_calculated_share(&mut self, share: SecretShare) {
        self.calculated_shares.insert(share.id, share);
    }

    pub fn send_masked_share(&self, id: u64) -> Option<SecretShareSend> {
        match self.calculated_shares.get(&id) {
            Some(share) => {
                return Some(SecretShareSend {
                    id: share.id,
                    share: share.share ^ share.mask,
                });
            }
            None => {}
        }

        match self.saved_shares.get(&id) {
            Some(share) => {
                return Some(SecretShareSend {
                    id: share.id,
                    share: share.share ^ share.mask,
                });
            }
            None => return None,
        }
    }
    pub fn send_unmasked_share(&self, id: u64) -> Option<SecretShareSend> {
        match self.saved_shares.get(&id) {
            Some(share) => {
                return Some(SecretShareSend {
                    id: share.id,
                    share: share.share,
                });
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::operation::and_operation;
    use crate::helpers::secret_share::generate_secret_share;
    #[test]
    fn test_node_creation() {
        let node = Node::new();

        assert!(node.saved_shares.is_empty());
        assert!(node.received_shares.is_empty());
        assert!(node.calculated_shares.is_empty());
    }

    #[test]
    fn test_three_nodes_secret_sharing() {
        // Create 3 nodes
        let mut node1 = Node::new();
        let mut node2 = Node::new();
        let mut node3 = Node::new();

        // Verify initial state
        assert_eq!(node1.saved_shares.len(), 0);
        assert_eq!(node1.received_shares.len(), 0);
        assert_eq!(node1.calculated_shares.len(), 0);

        let secret_share1 = generate_secret_share(0b101010);
        let secret_share2 = generate_secret_share(0b100010);
        let id1 = secret_share1[0].id;
        let id2 = secret_share2[0].id;

        node1.add_saved_share(secret_share1[0].clone());
        node2.add_saved_share(secret_share1[1].clone());
        node3.add_saved_share(secret_share1[2].clone());

        node1.add_saved_share(secret_share2[0].clone());
        node2.add_saved_share(secret_share2[1].clone());
        node3.add_saved_share(secret_share2[2].clone());

        assert_eq!(node1.saved_shares.len(), 2);
        assert_eq!(node2.saved_shares.len(), 2);
        assert_eq!(node3.saved_shares.len(), 2);

        // Now let assume I want to know and of this 2 values
        node2.add_received_share(node1.send_unmasked_share(id1).unwrap_or_default());
        node2.add_received_share(node1.send_unmasked_share(id2).unwrap_or_default());

        node3.add_received_share(node2.send_unmasked_share(id1).unwrap_or_default());
        node3.add_received_share(node2.send_unmasked_share(id2).unwrap_or_default());

        node1.add_received_share(node3.send_unmasked_share(id1).unwrap_or_default());
        node1.add_received_share(node3.send_unmasked_share(id2).unwrap_or_default());

        node1.add_calculated_share(and_operation(
            node1
                .saved_shares
                .get(&id1)
                .expect("Missing saved share for id1"),
            node1
                .saved_shares
                .get(&id2)
                .expect("Missing saved share for id2"),
            node1
                .received_shares
                .get(&id1)
                .expect("Missing received share for id1"),
            node1
                .received_shares
                .get(&id2)
                .expect("Missing received share for id2"),
            node1
                .saved_shares
                .get(&id1)
                .expect("Missing saved share for id1")
                .mask,
        ));

        node2.add_calculated_share(and_operation(
            node2
                .saved_shares
                .get(&id1)
                .expect("Missing saved share for id1"),
            node2
                .saved_shares
                .get(&id2)
                .expect("Missing saved share for id2"),
            node2
                .received_shares
                .get(&id1)
                .expect("Missing received share for id1"),
            node2
                .received_shares
                .get(&id2)
                .expect("Missing received share for id2"),
            node2
                .saved_shares
                .get(&id1)
                .expect("Missing saved share for id1")
                .mask,
        ));

        node3.add_calculated_share(and_operation(
            node3
                .saved_shares
                .get(&id1)
                .expect("Missing saved share for id1"),
            node3
                .saved_shares
                .get(&id2)
                .expect("Missing saved share for id2"),
            node3
                .received_shares
                .get(&id1)
                .expect("Missing received share for id1"),
            node3
                .received_shares
                .get(&id2)
                .expect("Missing received share for id2"),
            node3
                .saved_shares
                .get(&id1)
                .expect("Missing saved share for id1")
                .mask,
        ));

        let secret1 = node1.send_masked_share(id1 ^ id2).unwrap_or_default();
        let secret2 = node2.send_masked_share(id1 ^ id2).unwrap_or_default();
        let secret3 = node3.send_masked_share(id1 ^ id2).unwrap_or_default();

        assert_eq!(secret1.share ^ secret2.share ^ secret3.share, 0b100010)
    }
}
