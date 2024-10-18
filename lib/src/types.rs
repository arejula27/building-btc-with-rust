use crate::crypto::{PublicKey, Signature};
use crate::error::{BtcError, Result};
use crate::sha256::Hash;
use crate::util::MerkleRoot;
use crate::U256;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::u64;
use uuid::Uuid;
/// Blockchain is a chain of blocks
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Blockchain {
    /// A blockchain is a chain of blocks
    //a naive implementation would be a vector of blocks.
    pub blocks: Vec<Block>,
    pub utxos: HashMap<Hash, TransactionOutput>,
}

impl Blockchain {
    /// Constructor for the Blockchain type, by default it will be empty.
    pub fn new() -> Self {
        Blockchain {
            blocks: vec![],
            utxos: HashMap::new(),
        }
    }
    //As we are using a vector we added the block to the end of the vector.
    /// Add a block to the blockchain.
    pub fn add_block(&mut self, block: Block) -> Result<()> {
        //check if the blockchain is empty
        if self.blocks.is_empty() {
            //if this is the first block check if the
            //block's prev_block hash is all zeros
            if block.header.prev_block_hash != Hash::zero() {
                print!("zero hash");
                return Err(BtcError::InvalidBlock);
            } else {
                // if this is not the first block, check if the
                // block's prev_block_hash is the hash of the last block
                let last_block = self.blocks.last().unwrap();
                if block.header.prev_block_hash != last_block.hash() {
                    println!("prev hash is wrong");
                    return Err(BtcError::InvalidBlock);
                }
                if !block.header.hash().matches_target(block.header.target) {
                    println!("does not match target");
                    return Err(BtcError::InvalidBlock);
                }

                //check of the block's merkle root is correct
                let callculated_merkle_root =
                    MerkleRoot::calculate(&block.transactions);
                if callculated_merkle_root != block.header.merkle_root {
                    println!("invalid merkle root");
                    return Err(BtcError::InvalidMerkleRoot);
                }
                // check if the block's timestamp is after the
                // last block's timestamp
                if block.header.timestamp <= last_block.header.timestamp {
                    println!("invalid block timestamp");
                    return Err(BtcError::InvalidBlock);
                }
                //Verify all transactions in the block
                block.verify_transactions(self.block_height(), &self.utxos)?;
            }
        }
        self.blocks.push(block);
        Ok(())
    }

    //Rebuild UTXO set from the blockchain
    pub fn rebuild_utxos(&mut self) {
        for block in &self.blocks {
            for transaction in &block.transactions {
                // If a transaction output is used as input, the
                // output must be removed from the UTXO set
                for input in &transaction.inputs {
                    self.utxos.remove(&input.prev_transaction_output_hash);
                }
                // add all new transactions outputs to the UTXO set
                for output in transaction.outputs.iter() {
                    self.utxos.insert(transaction.hash(), output.clone());
                }
            }
        }
    }
    pub fn block_height(&self) -> u64 {
        self.blocks.len() as u64
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
    //Verify all transactions in the block
    //A transactions must:
    // - have the input from a UTXO
    // - do not spend the same input twice in the block
    // -  has a valid signature
    // - has a output value less or equal than the input value
    pub fn verify_transactions(
        &self,
        predicted_block_height: u64,
        utxos: &HashMap<Hash, TransactionOutput>,
    ) -> Result<()> {
        let mut inputs: HashMap<Hash, TransactionOutput> = HashMap::new();
        //reject completely empty blocks
        if self.transactions.is_empty() {
            return Err(BtcError::InvalidBlock);
        }
        //verify coinbase transaction
        self.verify_coinbase_transaction(predicted_block_height, utxos)?;

        for transaction in &self.transactions {
            let mut input_value = 0;
            let mut output_value = 0;
            for input in &transaction.inputs {
                let prev_output =
                    utxos.get(&input.prev_transaction_output_hash);
                //If the transaction inputs does not come from an
                //UTXO it is not valid
                if prev_output.is_none() {
                    return Err(BtcError::InvalidTransaction);
                }
                let prev_output = prev_output.unwrap();
                //Prevents same-block double-spending, if a input is already in the
                //inputs hash maps it means that a previous transaction in the same
                //block comes from the same input
                if inputs.contains_key(&input.prev_transaction_output_hash) {
                    return Err(BtcError::InvalidTransaction);
                }

                // check if the signature is valid
                if !input.signature.verify(
                    &input.prev_transaction_output_hash,
                    &prev_output.pubkey,
                ) {
                    return Err(BtcError::InvalidSignature);
                }

                // Update the inputs values
                input_value += prev_output.value;
                //add the input to the input hashmap
                inputs.insert(
                    input.prev_transaction_output_hash,
                    prev_output.clone(),
                );
            }
            //update the output value
            for output in &transaction.outputs {
                output_value += output.value;
            }
            // It is fine for output value to be less than input value
            // as the difference is the fee for the miner
            // But we must be sure that the output is the same or less
            // than the input value
            if input_value < output_value {
                return Err(BtcError::InvalidTransaction);
            }
        }
        Ok(())
    }
    pub fn verify_coinbase_transaction(
        &self,
        predicted_block_height: u64,
        utxos: &HashMap<Hash, TransactionOutput>,
    ) -> Result<()> {
        //Coinbase transaction is the first transation in the block
        let coinbase_transaction = &self.transactions[0];
        //The coinbase transaction generates new BTC
        // it must not have any input
        if coinbase_transaction.inputs.len() != 0 {
            return Err(BtcError::InvalidTransaction);
        }
        //It must always generate new BTC, outputs can not be 0
        if coinbase_transaction.outputs.len() == 0 {
            return Err(BtcError::InvalidTransaction);
        }
        //get the value of the whole block fee
        let miner_fees = self.calculate_miner_fees(utxos)?;
        //get the value of the expected new bitcoin minned
        let block_reward = crate::INITIAL_REWARD * 10u64.pow(8)
            / 2u64
                .pow((predicted_block_height / crate::HALVING_INTERVAL) as u32);
        let total_coinbase_outputs: u64 = coinbase_transaction
            .outputs
            .iter()
            .map(|output| output.value)
            .sum();
        // if the coinbase value does not match the expected it is an invalid coinbase
        // transaction
        if total_coinbase_outputs != block_reward + miner_fees {
            return Err(BtcError::InvalidTransaction);
        }
        Ok(())
    }

    pub fn calculate_miner_fees(
        &self,
        utxos: &HashMap<Hash, TransactionOutput>,
    ) -> Result<u64> {
        let mut inputs: HashMap<Hash, TransactionOutput> = HashMap::new();
        let mut outputs: HashMap<Hash, TransactionOutput> = HashMap::new();

        //check every transaction after coinbase
        for transaction in self.transactions.iter().skip(1) {
            for input in &transaction.inputs {
                //inputs do not contain the values of the outputs so we need to
                //match inputs to outputs
                let prev_output =
                    utxos.get(&input.prev_transaction_output_hash);
                if prev_output.is_none() {
                    return Err(BtcError::InvalidTransaction);
                }
                let prev_output = prev_output.unwrap();
                if inputs.contains_key(&input.prev_transaction_output_hash) {
                    return Err(BtcError::InvalidTransaction);
                }
                //we populate the hashmap with the outputs hash and the transaction
                //outputs which produce the inputs of the current transactions.
                inputs.insert(
                    input.prev_transaction_output_hash,
                    prev_output.clone(),
                );
            }
            for output in &transaction.outputs {
                //Avoid adding the same output twice
                if outputs.contains_key(&output.hash()) {
                    return Err(BtcError::InvalidTransaction);
                }
                outputs.insert(output.hash(), output.clone());
            }
        }
        let input_value: u64 = inputs.values().map(|output| output.value).sum();
        let output_value: u64 =
            outputs.values().map(|output| output.value).sum();
        //The fee is the difference between the input value and the output value of all
        // transactions
        Ok(input_value - output_value)
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
    pub fn hash(&self) -> Hash {
        Hash::hash(self)
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
