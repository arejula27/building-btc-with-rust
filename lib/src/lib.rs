use serde::{Deserialize, Serialize};
use uint::construct_uint;
construct_uint! {
    ///Construct a 256-bit unsigned integer.
    /// consist of 4 64-bit words.
    #[derive(Serialize, Deserialize)]
    pub struct U256(4);
}
pub mod crypto;
pub mod sha256;
pub mod types;
pub mod util;
