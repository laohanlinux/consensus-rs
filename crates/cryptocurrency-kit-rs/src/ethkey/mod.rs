pub mod secret;
pub mod error;
pub mod crypto;
pub mod random;
pub mod keypair;
pub mod keccak;
pub mod signature;

pub use self::error::Error;
pub use self::secret::Secret;
pub use self::keypair::{KeyPair, public_to_address};
pub use self::random::Random;
pub use self::signature::{sign, sign_bytes, verify_public, verify_address, recover, recover_bytes, Signature, SIGNATURE_SIZE};


use secp256k1::Secp256k1;
use ethereum_types::H256;
pub use ethereum_types::{Address, Public};
pub type Message = H256; // 256 / 8 = 32byte

lazy_static! {
    pub static ref SECP256K1: Secp256k1 = Secp256k1::new();
}

/// Uninstantiatable error type for infallible generators.
#[derive(Debug)]
pub enum Void {}

/// Generates new keypair.
pub trait Generator {
    type Error;

    /// Should be called to generate new kaypair
    fn generate(&mut self) -> Result<KeyPair, Self::Error>;
}