use eip712::Eip712Domain;
pub use eip_712_derive as eip712;
use ethers::{abi::Address, contract::abigen};
use std::{
    fmt::{self, Debug},
    str::FromStr,
};

abigen!(
    Subscriptions,
    "../contract/build/Subscriptions.abi",
    event_derives(serde::Deserialize, serde::Serialize);
    IERC20,
    "../contract/build/IERC20.abi",
    event_derives(serde::Deserialize, serde::Serialize);
);

pub struct Ticket {
    pub user: eip_712_derive::Bytes20,
    pub nonce: eip_712_derive::Bytes8,
}

impl Ticket {
    pub fn eip712_domain(chain_id: u64, contract_address: Address) -> Eip712Domain {
        let mut chain_id_bytes = [0_u8; 32];
        chain_id_bytes[24..].clone_from_slice(&chain_id.to_be_bytes());
        Eip712Domain {
            name: "Graph Subscriptions".to_string(),
            version: "0".to_string(),
            chain_id: eip712::U256(chain_id_bytes),
            verifying_contract: eip712::Address(contract_address.0),
            salt: [42_u8; 32],
        }
    }
}

impl eip712::StructType for Ticket {
    const TYPE_NAME: &'static str = "Ticket";
    fn visit_members<T: eip712::MemberVisitor>(&self, visitor: &mut T) {
        visitor.visit("user", &self.user);
        visitor.visit("nonce", &self.nonce);
    }
}

#[derive(Clone)]
pub struct Url(pub url::Url);

impl FromStr for Url {
    type Err = url::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        url::Url::from_str(s).map(Self)
    }
}

impl Debug for Url {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.as_str())
    }
}
