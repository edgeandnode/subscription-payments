use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};

#[derive(Debug, Default)]
pub struct SubscriptionTiers(Vec<SubscriptionTier>);

#[serde_as]
#[derive(Clone, Debug, Default, Deserialize)]
pub struct SubscriptionTier {
    /// Payment rate from the subcription contract.
    #[serde_as(as = "DisplayFromStr")]
    pub payment_rate: u128,
    /// Maximum query rate allowed, in queries per minute.
    pub queries_per_minute: u32,
    /// Maximum queries per month.
    #[serde(default)]
    pub monthly_query_limit: Option<u64>,
}

impl SubscriptionTiers {
    pub fn new(mut tiers: Vec<SubscriptionTier>) -> Self {
        tiers.sort_by_key(|t| t.payment_rate);
        Self(tiers)
    }

    pub fn tier_for_rate(&self, sub_rate: u128) -> SubscriptionTier {
        self.0
            .iter()
            .find(|tier| tier.payment_rate <= sub_rate)
            .cloned()
            .unwrap_or_default()
    }
}

impl From<Vec<SubscriptionTier>> for SubscriptionTiers {
    fn from(tiers: Vec<SubscriptionTier>) -> Self {
        Self::new(tiers)
    }
}
