use anyhow::{anyhow, ensure};
use chrono::{DateTime, NaiveDateTime, Utc};
use eip712::{DomainSeparator, Eip712Domain, PrivateKey};
pub use eip_712_derive as eip712;
use ethers::{
    abi::Address,
    contract::abigen,
    types::{RecoveryMessage, Signature},
};
use serde::{Deserialize, Serialize};

abigen!(
    Subscriptions,
    "../contract/build/Subscriptions.abi",
    event_derives(serde::Deserialize, serde::Serialize);
    IERC20,
    "../contract/build/IERC20.abi",
    event_derives(serde::Deserialize, serde::Serialize);
);

#[derive(Deserialize, Serialize)]
pub struct TicketPayload {
    /// Unique identifier.
    pub id: [u8; 8],
    /// Address associated with the secret key used to sign the ticket.
    pub signer: [u8; 20],
    /// Disambiguates when an authorized signer is also a user. Defaults to `signer` when omitted.
    pub user: Option<[u8; 20]>,
    // /// Maximum uses for tickets with matching identifiers. Defaults to 1 when omitted.
    // pub max_uses: Option<u64>,
    // /// Unix timestamp after which the ticket is invalid.
    // pub expiration: Option<u64>,
}

impl eip712::StructType for TicketPayload {
    const TYPE_NAME: &'static str = "Ticket";
    fn visit_members<T: eip712::MemberVisitor>(&self, visitor: &mut T) {
        visitor.visit("id", &self.id);
        visitor.visit("signer", &self.signer);
        visitor.visit("user", &self.user.unwrap_or(self.signer));
    }
}

impl TicketPayload {
    pub fn verify(
        &self,
        domain_separator: &DomainSeparator,
        signature: &[u8; 65],
    ) -> anyhow::Result<Address> {
        let hash = eip712::sign_hash(domain_separator, self);
        let signature = Signature {
            r: signature[0..32].into(),
            s: signature[32..64].into(),
            v: signature[64].into(),
        };
        let recovered_signer = signature.recover(RecoveryMessage::Hash(hash.into()))?;
        ensure!(&recovered_signer.0 == &self.signer);
        Ok(self.signer.into())
    }

    pub fn encode(
        &self,
        domain_separator: &DomainSeparator,
        key: &PrivateKey,
    ) -> anyhow::Result<Vec<u8>> {
        let (sig, r) = self.sign_hash(domain_separator, key)?;
        let mut buf = Vec::<u8>::new();
        ciborium::ser::into_writer(self, &mut buf)?;
        buf.append(&mut sig.into());
        buf.push(r);
        Ok(buf)
    }

    pub fn sign_hash(
        &self,
        domain_separator: &DomainSeparator,
        key: &PrivateKey,
    ) -> anyhow::Result<([u8; 64], u8)> {
        Ok(eip712::sign_typed(domain_separator, self, key)?)
    }

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
