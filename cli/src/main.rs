use anyhow::{ensure, Context, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use ethers::{abi::Address, prelude::*};
use graph_subscriptions::{eip712, Subscription, Subscriptions, Ticket, IERC20};
use rand::{thread_rng, RngCore as _};
use serde_json::json;
use std::{io::stdin, str::FromStr as _, sync::Arc};
use toolshed::url::Url;

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
        #[arg(long)]
        start: Option<DateTime<Utc>>,
        #[arg(long)]
        end: DateTime<Utc>,
        #[arg(long)]
        rate: u128,
    },
    Unsubscribe,
    Collect,
    Ticket {
        #[arg(long, help = "random by default")]
        nonce: Option<u64>,
        // #[arg(long, help = "ticket expiration")]
        // expiration: Option<DateTime<Utc>>,
        // #[arg(long, help = "maximum uses")]
        // max_uses: Option<u64>,
    },
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
            let active_sub: Subscription = subscriptions
                .subscriptions(wallet.address())
                .await?
                .try_into()?;
            println!("{active_sub:?}");
        }

        Commands::Subscribe { start, end, rate } => {
            let start = start.unwrap_or_else(Utc::now);
            eprintln!("start: {start}\n  end: {end}");
            ensure!(start < end);
            let duration: u64 = (end - start)
                .num_seconds()
                .try_into()
                .context("invalid sub duration")?;
            eprintln!("duration: {duration} s");

            let call = token.approve(subscriptions.address(), U256::from(rate) * duration);
            eprintln!("tx: {}", call.tx.data().unwrap());
            let receipt = client.send_transaction(call.tx, None).await?.await?;
            let status = receipt
                .and_then(|receipt| Some(receipt.status?.as_u64()))
                .unwrap_or(0);
            eprintln!("approve status: {}", status);
            ensure!(status == 1, "failed to approve token amount");

            let call = subscriptions.subscribe(
                wallet.address(),
                start.timestamp() as u64,
                end.timestamp() as u64,
                rate,
            );
            eprintln!("tx: {}", call.tx.data().unwrap());
            let receipt = client.send_transaction(call.tx, None).await?.await?;
            let status = receipt
                .and_then(|receipt| Some(receipt.status?.as_u64()))
                .unwrap_or(0);
            eprintln!("subscribe status: {}", status);
            ensure!(status == 1, "failed to subscribe");
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
            let domain = Ticket::eip712_domain(opt.chain_id, subscriptions.address());
            let domain_separator = eip712::DomainSeparator::new(&domain);

            let ticket = Ticket {
                user: wallet.address().0,
                nonce: nonce
                    .unwrap_or_else(|| thread_rng().next_u64())
                    .to_be_bytes(),
            };

            let (rs, v) = eip712::sign_typed(
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

            println!(
                "{}",
                json!({
                    "r": signature.r,
                    "s": signature.s,
                    "v": signature.v,
                    "user": format!("0x{}", hex::encode(ticket.user)),
                    "nonce": format!("0x{}",hex::encode(ticket.nonce)),
                })
            );
        }
    }

    Ok(())
}
