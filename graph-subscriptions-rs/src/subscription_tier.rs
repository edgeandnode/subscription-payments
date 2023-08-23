use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

#[derive(Clone, Debug, Default)]
pub struct SubscriptionTiers(Vec<SubscriptionTier>);

#[serde_as]
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
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
            .rev()
            .find(|tier| tier.payment_rate <= sub_rate)
            .cloned()
            .unwrap_or_default()
    }

    pub fn find_next_tier(&self, sub_rate: u128) -> Option<SubscriptionTier> {
        self.0
            .iter()
            .find(|tier| tier.payment_rate > sub_rate)
            .cloned()
    }
}

impl From<Vec<SubscriptionTier>> for SubscriptionTiers {
    fn from(tiers: Vec<SubscriptionTier>) -> Self {
        Self::new(tiers)
    }
}

impl AsRef<[SubscriptionTier]> for SubscriptionTiers {
    fn as_ref(&self) -> &[SubscriptionTier] {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use super::{SubscriptionTier, SubscriptionTiers};

    #[test]
    fn tier_for_rate() {
        let tiers = SubscriptionTiers::from(vec![
            SubscriptionTier {
                payment_rate: 100,
                queries_per_minute: 1,
                monthly_query_limit: None,
            },
            SubscriptionTier {
                payment_rate: 200,
                queries_per_minute: 2,
                monthly_query_limit: None,
            },
            SubscriptionTier {
                payment_rate: 300,
                queries_per_minute: 3,
                monthly_query_limit: None,
            },
        ]);

        assert_eq!(tiers.tier_for_rate(99).queries_per_minute, 0);
        assert_eq!(tiers.tier_for_rate(100).queries_per_minute, 1);
        assert_eq!(tiers.tier_for_rate(101).queries_per_minute, 1);
        assert_eq!(tiers.tier_for_rate(199).queries_per_minute, 1);
        assert_eq!(tiers.tier_for_rate(200).queries_per_minute, 2);
        assert_eq!(tiers.tier_for_rate(201).queries_per_minute, 2);
        assert_eq!(tiers.tier_for_rate(299).queries_per_minute, 2);
        assert_eq!(tiers.tier_for_rate(300).queries_per_minute, 3);
        assert_eq!(tiers.tier_for_rate(500).queries_per_minute, 3);
    }
}
