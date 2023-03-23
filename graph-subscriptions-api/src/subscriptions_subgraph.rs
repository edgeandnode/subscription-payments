use std::{
    collections::HashMap,
    str::FromStr,
    sync::Arc,
    time::{Duration, SystemTime},
};

use chrono::{DateTime, NaiveDateTime, Utc};
use eventuals::{Eventual, EventualExt as _, EventualWriter, Ptr};
use serde::{de::Error, Deserialize, Deserializer};
use tokio::sync::Mutex;
use toolshed::bytes::Address;

use crate::subgraph_client;

#[derive(Clone)]
pub struct Subscription {
    pub signers: Vec<Address>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AuthorizedSigner {
    pub signer: Address,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Address,
    #[serde(default)]
    pub authorized_signers: Vec<AuthorizedSigner>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ActiveSubscription {
    pub user: User,
    #[serde(deserialize_with = "deserialize_datetime_utc")]
    pub start: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_datetime_utc")]
    pub end: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_u128")]
    pub rate: u128,
}

fn deserialize_datetime_utc<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let input = String::deserialize(deserializer)?;
    let timestamp = input.parse::<u64>().map_err(D::Error::custom)?;
    NaiveDateTime::from_timestamp_opt(timestamp.try_into().map_err(D::Error::custom)?, 0)
        .ok_or_else(|| D::Error::custom("invalid timestamp"))
        .map(|t| DateTime::<Utc>::from_utc(t, Utc))
}

fn deserialize_u128<'de, D>(deserializer: D) -> Result<u128, D::Error>
where
    D: Deserializer<'de>,
{
    let input = String::deserialize(deserializer)?;
    u128::from_str(&input).map_err(D::Error::custom)
}

pub struct Client {
    subgraph_client: subgraph_client::Client,
    subscriptions: EventualWriter<Ptr<HashMap<Address, Subscription>>>,
}

impl Client {
    pub fn create(
        subgraph_client: subgraph_client::Client,
    ) -> Eventual<Ptr<HashMap<Address, Subscription>>> {
        let (subscriptions_tx, subscriptions_rx) = Eventual::new();
        let client = Arc::new(Mutex::new(Client {
            subgraph_client,
            subscriptions: subscriptions_tx,
        }));

        eventuals::timer(Duration::from_secs(30))
            .pipe_async(move |_| {
                let client = client.clone();
                async move {
                    let mut client = client.lock().await;
                    if let Err(poll_active_subscriptions_err) =
                        client.poll_active_subscriptions().await
                    {
                        tracing::error!(%poll_active_subscriptions_err);
                    }
                }
            })
            .forever();

        subscriptions_rx
    }

    async fn poll_active_subscriptions(&mut self) -> Result<(), String> {
        // Serve queries for subscriptions that end 10 minutes ago and later.
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let active_sub_end = (timestamp / 1000) - (60 * 10);

        let query = format!(
            r#"
            activeSubscriptions(
                block: $block
                orderBy: id, orderDirection: asc
                first: $first
                where: {{
                    id_gt: $last
                    end_gte: {active_sub_end}
                }}
            ) {{
                id
                user {{
                    id
                    authorizedSigners {{
                        signer
                    }}
                }}
                start
                end
                rate
            }}
          "#,
        );
        let active_subscriptions_response = self
            .subgraph_client
            .paginated_query::<ActiveSubscription>(&query)
            .await?;
        if active_subscriptions_response.is_empty() {
            return Err("Discarding empty update (active_subscriptions)".to_string());
        }

        let subscriptions_map = active_subscriptions_response
            .into_iter()
            .map(|active_sub| {
                let user = active_sub.user;
                let signers = user
                    .authorized_signers
                    .into_iter()
                    .map(|signer| signer.signer)
                    .chain([user.id]);
                let sub = Subscription {
                    signers: signers.collect(),
                };
                (user.id, sub)
            })
            .collect();
        self.subscriptions.write(Ptr::new(subscriptions_map));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::ensure;
    use serde_json::json;

    use super::*;

    #[test]
    fn should_parse_active_subscriptions_query() -> anyhow::Result<()> {
        let result = serde_json::from_str::<ActiveSubscription>(
            &json!({
                "user": {
                    "id": "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266",
                    "authorizedSigners": [
                        {
                            "signer": "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266"
                        }
                    ],
                },
                "start": "1676507163",
                "end": "1676507701",
                "rate": "100000000000000",
            })
            .to_string(),
        );
        ensure!(result.is_ok(), "failed to parse example: {:?}", result);
        Ok(())
    }
}
