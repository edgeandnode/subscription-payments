use anyhow::anyhow;
use chrono::{DateTime, NaiveDateTime, Utc};
use eip712::Eip712Domain;
pub use eip_712_derive as eip712;
use ethers::{abi::Address, contract::abigen};

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

#[derive(Debug)]
pub struct Subscription {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub rate: u128,
}

impl TryFrom<(u64, u64, u128)> for Subscription {
    type Error = anyhow::Error;
    fn try_from(from: (u64, u64, u128)) -> Result<Self, Self::Error> {
        let (start, end, rate) = from;
        let to_datetime = |t: u64| {
            NaiveDateTime::from_timestamp_opt(t.try_into()?, 0)
                .ok_or_else(|| anyhow!("invalid timestamp"))
                .map(|t| DateTime::<Utc>::from_utc(t, Utc))
        };
        let start = to_datetime(start)?;
        let end = to_datetime(end)?;
        Ok(Self { start, end, rate })
    }
}
