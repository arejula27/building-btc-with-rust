use crate::sha256::Hash;
use crate::types::Transaction;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MerkleRoot(Hash);

impl MerkleRoot {
    /// Calculates the Merkle root from a slice of transactions.
    ///
    /// # Arguments
    ///
    /// * `transactions` - A slice of `Transaction` objects, for which the Merkle root will be computed.
    ///
    /// # Returns
    ///
    /// * `MerkleRoot` - The calculated Merkle root of the transactions.
    ///
    /// # How it works
    ///
    /// 1. The function starts by hashing each individual transaction and placing these hashes in the `layer` vector.
    /// 2. It then iteratively combines pairs of hashes from the current layer into a new layer, by hashing the concatenation
    ///    of two hashes at a time (or duplicating the last hash if the number of hashes is odd).
    /// 3. This process continues until only a single hash remains, which represents the root of the Merkle tree.
    ///
    /// The Merkle root is then returned as the root hash.
    pub fn calculate(trasactions: &[Transaction]) -> MerkleRoot {
        let mut layer: Vec<Hash> = vec![];
        // Hash each transaction and add it to the first layer of the tree.
        for trasaction in trasactions {
            layer.push(Hash::hash(trasaction));
        }
        // Combine pairs of hashes from the current layer into a new layer.
        while layer.len() > 1 {
            let mut new_layer = vec![];
            //Each pair of hashes is concatenated and hashed to create a new hash.
            for pair in layer.chunks(2) {
                let left = pair[0];
                //if there is no right, use the left hash again
                let right = pair.get(1).unwrap_or(&pair[0]);
                new_layer.push(Hash::hash(&[left, *right]));
            }
            layer = new_layer
        }
        MerkleRoot(layer[0])
    }
}
