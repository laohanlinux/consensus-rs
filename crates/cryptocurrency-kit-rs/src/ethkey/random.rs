use rand::rngs::OsRng;
use super::{Void, Generator, KeyPair, SECP256K1};

#[derive(Default)]
pub struct Random;

impl Generator for Random {
    type Error = ::std::io::Error;

    fn generate(&mut self) -> Result<KeyPair, Self::Error> {
        let mut rng = OsRng;
        match rng.generate() {
            Ok(pair) => Ok(pair),
            Err(void) => match void { }, // LLVM unreachable
        }
    }
}

impl Random {
    pub fn generate_keypair() -> KeyPair {
        let mut rd = Random::default();
        rd.generate().unwrap()
    }
}

impl Generator for OsRng {
    type Error = Void;

    fn generate(&mut self) -> Result<KeyPair, Self::Error> {
        let (sec, publ) = SECP256K1.generate_keypair(self)
            .expect("context always created with full capabilities; qed");
        Ok(KeyPair::from_keypair(sec, publ))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Write};
    #[test]
    fn bench_generate() {
        (0..100).for_each(|i: i32|{
            let keypair = Random::generate_keypair();
            writeln!(io::stdout(), "-----------\n{} \n{}", i, keypair).unwrap();
        })
    }
}