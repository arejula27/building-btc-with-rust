use crate::crypto::{PublicKey, Signature};
use crate::sha256::Hash;
use crate::util::MerkleRoot;
use crate::U256;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
/// Blockchain is a chain of blocks
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Blockchain {
    /// A blockchain is a chain of blocks
    //a naive implementation would be a vector of blocks.
    pub blocks: Vec<Block>,
}

impl Blockchain {
    /// Constructor for the Blockchain type, by default it will be empty.
    pub fn new() -> Self {
        Blockchain { blocks: vec![] }
    }
    //As we are using a vector we added the block to the end of the vector.
    /// Add a block to the blockchain.
    pub fn add_block(&mut self, block: Block) {
        self.blocks.push(block);
    }
}

/// A block is a collection of transactions with a header.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block {
    /// The header of the block
    pub header: BlockHeader,
    /// The transactions in the block
    pub transactions: Vec<Transaction>,
}
impl Block {
    pub fn new(header: BlockHeader, transactions: Vec<Transaction>) -> Self {
        Block {
            // in the new function if the parameter has the same name
            // as the field you can write just once instead of field
            // Block{header:header, transaction: transaction}
            header,
            transactions,
        }
        //this allows the function to be unimpemented but will crash at
        //runtime
    }
    pub fn hash(&self) -> Hash {
        //this allows the function to be unimpemented but will crash at
        //runtime
        Hash::hash(self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlockHeader {
    ///the time when the block was created.
    pub timestamp: DateTime<Utc>,
    /// number only used once, we increment it to mine the block
    pub nonce: u64,
    /// the hash of the previous block in the chain
    pub prev_block_hash: Hash,
    /// the hash of the Merkñe tree root derived from all of the transactions in this block.
    /// This ensures that all transactions are accounted for and unalterable without changing
    /// the header
    pub merkle_root: MerkleRoot,
    ///A number, which has to be higher than the hash of this block for it to be considered valid
    pub target: U256,
}

impl BlockHeader {
    pub fn new(
        timestamp: DateTime<Utc>,
        nonce: u64,
        prev_block_hash: Hash,
        merkle_root: MerkleRoot,
        target: U256,
    ) -> Self {
        BlockHeader {
            timestamp,
            nonce,
            prev_block_hash,
            merkle_root,
            target,
        }
    }
    pub fn hash(&self) -> ! {
        unimplemented!()
    }
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    pub inputs: Vec<TransactionInput>,
    pub outputs: Vec<TransactionOutput>,
}

impl Transaction {
    pub fn new(
        inputs: Vec<TransactionInput>,
        outputs: Vec<TransactionOutput>,
    ) -> Self {
        Transaction { inputs, outputs }
    }
    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionInput {
    /// Hash of the transaction output that we are going to use as input/// Real bitcoin uses a slightly different scheme - it stores the
    /// previous transaction hash, and the index of the output in
    /// that transaction.
    pub prev_transaction_output_hash: Hash,
    /// Signature of the user which proves they can expend the output
    /// of the previous transaction. In the real implementation of bitcoin it is required a
    /// script field instead just the pubkey. The main implementation of bitcoin can do many things in the script fields, but we are fine with a much simpler solution, where you can only send sats to a recipient and nothing else.
    pub signature: Signature,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransactionOutput {
    /// Value of the transaction output
    pub value: u64,
    /// Identifier of the transaction output
    pub unique_id: Uuid,
    /// Pubkey of the recipient, the signature of the corresponding
    /// private key must be used for signing the following transaction
    pub pubkey: PublicKey,
}
impl TransactionOutput {
    pub fn hash(&self) -> Hash {
        Hash::hash(self)
    }
}
