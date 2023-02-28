pub use eip_712_derive as eip712;

use anyhow::{anyhow, ensure, Context};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine as _};
use chrono::{DateTime, NaiveDateTime, Utc};
use eip712::{DomainSeparator, Eip712Domain, PrivateKey};
use ethers::{
    abi::Address,
    contract::abigen,
    types::{RecoveryMessage, Signature},
};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none, Bytes, FromInto};

abigen!(
    Subscriptions,
    "../contract/build/Subscriptions.abi",
    event_derives(serde::Deserialize, serde::Serialize);
    IERC20,
    "../contract/build/IERC20.abi",
    event_derives(serde::Deserialize, serde::Serialize);
);

// This is necessary intermediary to get the Address wrapper type over bytes to serialize &
// deserialize as bytes in the CBOR representation.
// See https://github.com/jonasbb/serde_with/discussions/557
#[serde_as]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
struct AddressBytes(#[serde_as(as = "Bytes")] [u8; 20]);
#[rustfmt::skip]
impl From<AddressBytes> for Address { fn from(value: AddressBytes) -> Self { value.0.into() } }
#[rustfmt::skip]
impl From<Address> for AddressBytes { fn from(value: Address) -> Self { Self(value.0) } }

#[serde_as]
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TicketPayload {
    /// Unique identifier, used in conjunction with additional options such as `max_uses`.
    pub id: u64,
    /// Address associated with the secret key used to sign the ticket.
    #[serde_as(as = "FromInto<AddressBytes>")]
    pub signer: Address,
    /// Disambiguates when an authorized signer is also a user. Defaults to `signer` when omitted.
    #[serde_as(as = "Option<FromInto<AddressBytes>>")]
    pub user: Option<Address>,
    /// Optional nice name.
    pub name: Option<String>,
    // /// Maximum uses for tickets with matching identifiers. Defaults to 1 when omitted.
    // pub max_uses: Option<u64>,
    // /// Unix timestamp after which the ticket is invalid.
    // pub expiration: Option<u64>,
}

impl eip712::StructType for TicketPayload {
    const TYPE_NAME: &'static str = "Ticket";
    fn visit_members<T: eip712::MemberVisitor>(&self, visitor: &mut T) {
        visitor.visit("id", &self.id.to_be_bytes());
        visitor.visit("signer", &eip712::Address(self.signer.0));
        visitor.visit("user", &eip712::Address(self.user.unwrap_or(self.signer).0));
        visitor.visit("name", &self.name.clone().unwrap_or_default());
    }
}

impl TicketPayload {
    pub fn from_ticket_base64(
        ticket: &[u8],
        domain_separator: &DomainSeparator,
    ) -> anyhow::Result<(Self, [u8; 65])> {
        let ticket = base64::prelude::BASE64_URL_SAFE_NO_PAD
            .decode(ticket)
            .context("invalid base64 (URL, nopad)")?;

        let signature_start = ticket.len() - 65;
        let signature: &[u8; 65] = ticket[signature_start..]
            .try_into()
            .context("invalid signature")?;

        let payload: TicketPayload =
            serde_cbor_2::de::from_reader(&ticket[..signature_start]).context("invalid payload")?;
        let recovered_signer = payload
            .verify(domain_separator, signature)
            .context("failed to recover signer")?;
        ensure!(
            payload.signer == recovered_signer,
            "recovered signer does not match claim"
        );
        Ok((payload, *signature))
    }

    pub fn to_ticket_base64(
        &self,
        domain_separator: &DomainSeparator,
        key: &PrivateKey,
    ) -> anyhow::Result<String> {
        let ticket = self.encode(domain_separator, key)?;
        Ok(BASE64_URL_SAFE_NO_PAD.encode(ticket))
    }

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
        ensure!(&recovered_signer == &self.signer);
        Ok(self.signer)
    }

    pub fn encode(
        &self,
        domain_separator: &DomainSeparator,
        key: &PrivateKey,
    ) -> anyhow::Result<Vec<u8>> {
        let (sig, r) = self.sign_hash(domain_separator, key)?;
        let mut buf = serde_cbor_2::ser::to_vec(self)?;
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

#[cfg(test)]
#[test]
fn test_ticket() {
    let chain_id = 1337;
    let contract_address = "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512"
        .parse::<Address>()
        .unwrap();
    let domain = TicketPayload::eip712_domain(chain_id, contract_address);
    let domain_separator = eip712::DomainSeparator::new(&domain);

    let ticket = "omJpZBuJOINiTrw1jGZzaWduZXJU85_W5RqtiPb0zmq4gnJ5z_-5ImYCmn5n8Pp1OLU_iIu0et9VWzlF9Q8YhO_xcyORAj-paTEz0t52H0tvWtuxDK0ARF1BM209m57hTtVdp6JoM_luGw";
    let (payload, signature) =
        TicketPayload::from_ticket_base64(ticket.as_bytes(), &domain_separator).unwrap();
    println!("{:#?}", payload);
    println!("Signature({:?})", hex::encode(signature));
}
