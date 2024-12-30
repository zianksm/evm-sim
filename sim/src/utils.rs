use alloy::{
    hex::FromHex,
    primitives::utils::{ParseUnits, Unit},
};
use anyhow::Result;

use crate::evm::{Address, Uint256};
pub struct Utils;

impl Utils {
    pub fn address_try_from_string(mut s: String) -> Result<Address> {
        let address = s.parse::<Address>()?;

        Ok(address)
    }

    pub fn uint_try_from_string(s: &str) -> Result<Uint256> {
        let unit = ParseUnits::parse_units(s, Unit::ETHER)?;

        Ok(unit.into())
    }
}
