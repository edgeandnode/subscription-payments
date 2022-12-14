use anyhow::{ensure, Result};
use clap::{Parser, Subcommand};
use eip712::Eip712Domain;
use eip_712_derive as eip712;
use ethers::{abi::Address, contract::abigen, prelude::*};
use rand::{thread_rng, RngCore as _};
use std::{
    fmt::{self, Debug},
    io::{stdin, Cursor, Write},
    str::FromStr,
    sync::Arc,
};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Opt {
    #[arg(long, default_value = "http://localhost:8545")]
    provider: Url,
    #[arg(long, default_value = "1337")]
    chain_id: u64,
    #[arg(long, help = "subscriptions contract address")]
    subscriptions: Address,
    #[arg(long, help = "token contract address")]
    token: Address,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// show active subscription
    Active,
    Subscribe {
        #[arg(long, default_value = "0")]
        start_block: u64,
        #[arg(long)]
        end_block: u64,
        #[arg(long)]
        price_per_block: u128,
    },
    Unsubscribe,
    Collect,
    Ticket {
        #[arg(long, help = "random by default")]
        nonce: Option<u64>,
        // #[arg(long, help = "ticket expiration, in seconds since Unix epoch")]
        // expiration: Option<u64>,
        // #[arg(long, help = "maximum uses")]
        // max_uses: Option<u64>,
    },
}

abigen!(
    Subscriptions,
    "../contract/build/Subscriptions.abi",
    event_derives(serde::Deserialize, serde::Serialize);
    IERC20,
    "../contract/build/IERC20.abi",
    event_derives(serde::Deserialize, serde::Serialize);
);

pub struct Ticket {
    user: eip_712_derive::Bytes20,
    nonce: eip_712_derive::Bytes8,
}

impl eip712::StructType for Ticket {
    const TYPE_NAME: &'static str = "Ticket";
    fn visit_members<T: eip712::MemberVisitor>(&self, visitor: &mut T) {
        visitor.visit("user", &self.user);
        visitor.visit("nonce", &self.nonce);
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let opt = Opt::parse();
    eprintln!("{:#?}", opt);

    let provider = Arc::new(Provider::<Http>::try_from(opt.provider.0.as_str())?);
    let subscriptions = Subscriptions::new(opt.subscriptions, provider.clone());
    let token = IERC20::new(opt.token, provider.clone());

    let mut secret_key = String::new();
    stdin().read_line(&mut secret_key)?;
    let wallet = Wallet::from_str(secret_key.trim())?.with_chain_id(opt.chain_id);
    drop(secret_key);
    let client = SignerMiddleware::new(provider, wallet.clone());
    eprintln!("user: {}", wallet.address());

    let balance = token.balance_of(wallet.address()).await?;
    eprintln!("balance: {balance:?}");

    match opt.command {
        Commands::Active => {
            let active_sub = subscriptions.subscription(wallet.address()).await?;
            println!("{active_sub:?}");
        }

        Commands::Subscribe {
            start_block,
            end_block,
            price_per_block,
        } => {
            ensure!(start_block < end_block);
            let call = token.approve(
                subscriptions.address(),
                U256::from(price_per_block) * (end_block - start_block),
            );
            eprintln!("tx: {}", call.tx.data().unwrap());
            let receipt = client.send_transaction(call.tx, None).await?.await?;
            let status = receipt
                .and_then(|receipt| Some(receipt.status?.as_u64()))
                .unwrap_or(0);
            eprintln!("approve status: {}", status);
            ensure!(status == 1, "Failed to approve token amount");
            let call =
                subscriptions.subscribe(wallet.address(), start_block, end_block, price_per_block);
            eprintln!("tx: {}", call.tx.data().unwrap());
            let receipt = client.send_transaction(call.tx, None).await?.await?;
            let status = receipt
                .and_then(|receipt| Some(receipt.status?.as_u64()))
                .unwrap_or(0);
            eprintln!("subscribe status: {}", status);
            ensure!(status == 1, "Failed to subscribe");
        }

        Commands::Unsubscribe => {
            let call = subscriptions.unsubscribe();
            eprintln!("tx: {}", call.tx.data().unwrap());
            let receipt = client.send_transaction(call.tx, None).await?.await?;
            let status = receipt
                .and_then(|receipt| Some(receipt.status?.as_u64()))
                .unwrap_or(0);
            eprintln!("unsubscribe status: {}", status);
            ensure!(status == 1, "Failed to unsubscribe");
        }

        Commands::Collect => {
            let call = subscriptions.collect();
            eprintln!("tx: {}", call.tx.data().unwrap());
            let receipt = client.send_transaction(call.tx, None).await?.await?;
            let status = receipt
                .and_then(|receipt| Some(receipt.status?.as_u64()))
                .unwrap_or(0);
            eprintln!("collect status: {}", status);
            ensure!(status == 1, "Failed to collect");
        }

        Commands::Ticket { nonce } => {
            let mut chain_id = [0_u8; 32];
            chain_id[24..].clone_from_slice(&opt.chain_id.to_be_bytes());
            let domain = Eip712Domain {
                name: "Graph Subscriptions".to_string(),
                version: "0".to_string(),
                chain_id: eip712::U256(chain_id),
                verifying_contract: eip712::Address(subscriptions.address().0),
                salt: [42_u8; 32],
            };
            let domain_separator = eip712::DomainSeparator::new(&domain);

            let ticket = Ticket {
                user: wallet.address().0,
                nonce: nonce
                    .unwrap_or_else(|| thread_rng().next_u64())
                    .to_be_bytes(),
            };

            let (rs, v) = eip_712_derive::sign_typed(
                &domain_separator,
                &ticket,
                &wallet.signer().to_bytes().as_slice().try_into().unwrap(),
            )?;
            let signature = Signature {
                r: rs[0..32].try_into().unwrap(),
                s: rs[32..64].try_into().unwrap(),
                v: v as u64,
            };

            let sign_hash = eip712::sign_hash(&domain_separator, &ticket);
            ensure!(wallet.address() == signature.recover(sign_hash)?);

            let mut cursor = Cursor::new([0_u8; 28 + 65]);
            cursor.write_all(&ticket.user)?;
            cursor.write_all(&ticket.nonce)?;
            cursor.write_all(&signature.to_vec())?;
            ensure!(cursor.position() == cursor.get_ref().len() as u64);
            println!("0x{}", hex::encode(cursor.into_inner()))
        }
    }

    Ok(())
}

#[derive(Clone)]
struct Url(pub url::Url);

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
