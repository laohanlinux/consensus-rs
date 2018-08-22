use std::fmt::{self, Display};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HexBytes {
    inner: [u8; 32],
}

impl HexBytes {
    pub fn bytes(&self) -> &[u8;32] {
        &self.inner
    }

    pub fn string(&self) -> String {
        String::from_utf8_lossy(&self.inner).to_string()
    }
}

impl Display for HexBytes {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use std::fmt;
        writeln!(f, "{}", self.string()).unwrap();
        Ok(())
    }
}