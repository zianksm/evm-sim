use alloy::hex::FromHex;
use anyhow::Result;

use crate::evm::Address;
pub struct Utils;

impl Utils {
    pub fn address_try_from_string(mut s: String) -> Result<Address> {
        let address = s.parse::<Address>()?;

        Ok(address)
    }
}
