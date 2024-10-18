use core::panic;

use crate::U256;
use serde::{Deserialize, Serialize};
use sha256::digest;
use std::fmt;

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct Hash(U256);

impl Hash {
    //hash anything that can be serde Serialized via ciborium
    pub fn hash<T: serde::Serialize>(data: &T) -> Self {
        //create the buffer for storing the serialized value
        let mut serialized: Vec<u8> = vec![];
        if let Err(e) = ciborium::into_writer(data, &mut serialized) {
            panic!(
                "Failed to serialize data: {:?}. \
            This should not happen.",
                e
            )
        }
        // Obtain the SHA-256 hash of the serialized value.
        // The hash will be a string in hexadecimal format.
        // Example: "315f5bdb76d078c43b8ac0064e4a0164612b1fce77c869345bfc94c75894edd3"
        let hash = digest(&serialized);

        // Convert the hexadecimal string to a vector of bytes.
        // Each hexadecimal digit pair represents a byte.
        // For example, "31" becomes 49, and "5f" becomes 95.
        //[31,95,...]
        let hash_bytes = hex::decode(hash).unwrap();

        // Now we need to convert the Vector into a slice of 32 elements u8
        // - as_slice(): This method converts a Vec<u8> into a &[u8], which
        //              is a reference to the underlying array of bytes
        //              stored in the vector.

        let hash_slice: &[u8] = hash_bytes.as_slice();

        //Now we convert the slice into an array, this would fail in case the
        //slice is not 32 elements long, however the sha256 hash is always 32 bytes
        let hash_array: [u8; 32] = hash_slice.try_into().unwrap();
        Hash(U256::from(hash_array))
    }
    pub fn matches_target(&self, target: U256) -> bool {
        self.0 <= target
    }
    pub fn zero() -> Self {
        Hash(U256::zero())
    }

    pub fn as_bytes(&self) -> [u8; 32] {
        let mut bytes: Vec<u8> = vec![0; 32];
        self.0.to_little_endian(&mut bytes);

        bytes.as_slice().try_into().unwrap()
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}
