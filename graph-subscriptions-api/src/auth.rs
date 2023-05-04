use std::collections::HashMap;

use anyhow::{anyhow, ensure, Ok, Result};
use axum::http::{header::AUTHORIZATION, HeaderMap};
use eventuals::{Eventual, Ptr};
use graph_subscriptions::{TicketPayload, TicketVerificationDomain};
use thiserror::Error;
use toolshed::bytes::Address;

use crate::subscriptions_subgraph::UserSubscriptionWithSigners;

#[derive(Clone, Debug)]
pub struct TicketPayloadWrapper {
    pub ticket_payload: TicketPayload,
    pub active_subscription: UserSubscriptionWithSigners,
}

pub struct AuthHandler {
    pub subscriptions_domain: TicketVerificationDomain,
    pub subscriptions: Eventual<Ptr<HashMap<Address, UserSubscriptionWithSigners>>>,
}

impl AuthHandler {
    pub fn create(
        subscriptions_domain_separator: TicketVerificationDomain,
        subscriptions: Eventual<Ptr<HashMap<Address, UserSubscriptionWithSigners>>>,
    ) -> &'static Self {
        Box::leak(Box::new(Self {
            subscriptions_domain: subscriptions_domain_separator,
            subscriptions,
        }))
    }

    pub fn parse_auth_header(&self, headers: &HeaderMap) -> Result<TicketPayloadWrapper> {
        // grab the Authorization header out of the request headers map.
        // trim out "Bearer" from beginning of header, which should be in format: Beader {token}
        let raw_auth_header = headers
            .get(AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .unwrap_or(&"")
            .trim_start_matches("Bearer")
            .trim();
        // fail if no Authorization header Bearer token is present
        ensure!(
            !raw_auth_header.is_empty(),
            "No Authorization header found on request"
        );

        // parse the authorization header as an EIP-712 signed message
        let (payload, _) =
            TicketPayload::from_ticket_base64(&self.subscriptions_domain, &raw_auth_header)?;

        let user = Address(payload.user.unwrap_or(payload.signer).0);
        let active_subscription = self
            .subscriptions
            .value_immediate()
            .unwrap_or_default()
            .get(&user)
            .cloned()
            .ok_or_else(|| anyhow!("Subscription not found for user {}", user))?;
        let signer = Address(payload.signer.0);
        ensure!(
            (signer == user) || active_subscription.signers.contains(&signer),
            "Signer {} not authorized for user {}",
            signer,
            user,
        );

        Ok(TicketPayloadWrapper {
            ticket_payload: payload,
            active_subscription,
        })
    }
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Authorization Header not found on request, or is invalid")]
    Unauthenticated,
    #[error("Do not have access to the requested resource")]
    Unauthorized,
}

#[cfg(test)]
mod tests {
    use std::{ops::Add, str::FromStr};

    use axum::http::{header::AUTHORIZATION, HeaderMap};
    use chrono::{Duration, Utc};
    use ethers::types::{H160, U256};
    use eventuals::{Eventual, Ptr};
    use graph_subscriptions::{TicketPayload, TicketVerificationDomain};
    use toolshed::bytes::Address;

    use super::*;

    #[test]
    fn should_fail_if_no_authorization_header_present() {
        let subscriptions_domain = TicketVerificationDomain {
            contract: H160(
                Address::from_str("0x29f49a438c747e7Dd1bfe7926b03783E47f9447B")
                    .unwrap()
                    .0,
            ),
            chain_id: U256::from(421613),
        };
        let subscriptions = Eventual::from_value(Ptr::default());
        let handler = AuthHandler::create(subscriptions_domain, subscriptions);
        let headers = HeaderMap::new();

        let actual = handler.parse_auth_header(&headers);
        assert!(
            actual.is_err(),
            "should throw an error if not Authorization header present"
        );
    }

    #[test]
    fn should_fail_if_authorization_header_empty() {
        let subscriptions_domain = TicketVerificationDomain {
            contract: H160(
                Address::from_str("0x29f49a438c747e7Dd1bfe7926b03783E47f9447B")
                    .unwrap()
                    .0,
            ),
            chain_id: U256::from(421613),
        };
        let subscriptions = Eventual::from_value(Ptr::default());
        let handler = AuthHandler::create(subscriptions_domain, subscriptions);
        let mut headers = HeaderMap::new();
        headers.append(AUTHORIZATION, "".parse().unwrap());

        let actual = handler.parse_auth_header(&headers);
        assert!(
            actual.is_err(),
            "should throw an error if not Authorization header is empty"
        );
    }

    #[test]
    fn should_fail_if_authorization_header_not_bearer_token() {
        let subscriptions_domain = TicketVerificationDomain {
            contract: H160(
                Address::from_str("0x29f49a438c747e7Dd1bfe7926b03783E47f9447B")
                    .unwrap()
                    .0,
            ),
            chain_id: U256::from(421613),
        };
        let subscriptions = Eventual::from_value(Ptr::default());
        let handler = AuthHandler::create(subscriptions_domain, subscriptions);
        let mut headers = HeaderMap::new();
        headers.append(AUTHORIZATION, "invalid".parse().unwrap());

        let actual = handler.parse_auth_header(&headers);
        assert!(
            actual.is_err(),
            "should throw an error if not Authorization header is not valid bearer"
        );
    }

    #[test]
    fn should_fail_if_authorization_header_not_valid_eip_712_signed_message() {
        let subscriptions_domain = TicketVerificationDomain {
            contract: H160(
                Address::from_str("0x29f49a438c747e7Dd1bfe7926b03783E47f9447B")
                    .unwrap()
                    .0,
            ),
            chain_id: U256::from(421613),
        };
        let subscriptions = Eventual::from_value(Ptr::default());
        let handler = AuthHandler::create(subscriptions_domain, subscriptions);
        let mut headers = HeaderMap::new();
        headers.append(AUTHORIZATION, "Bearer invalid".parse().unwrap());

        let actual = handler.parse_auth_header(&headers);
        assert!(
            actual.is_err(),
            "should throw an error if not Authorization header is not valid EIP-712 signed message"
        );
    }

    #[test]
    fn should_fail_if_no_subscription_found_for_user() {
        let chain_id = 1337;
        let contract_address = "0xe7f1725E7734CE288F8367e1Bb143E90bb3F0512"
            .parse::<Address>()
            .unwrap();
        let domain_separator = TicketVerificationDomain {
            contract: H160(contract_address.0),
            chain_id: U256::from(chain_id),
        };
        let subscriptions = Eventual::from_value(Ptr::default());
        let handler = AuthHandler::create(domain_separator, subscriptions);
        let mut headers = HeaderMap::new();
        let ticket = "o2RuYW1lTnRlc3RfYXBpX2tleV8xZnNpZ25lclTBQrzwQKv5NwPAPazwLFS0DaDt63FhbGxvd2VkX3N1YmdyYXBoc1gsM25YZkszUmJGcmo2bWhrR2RvS1Jvd0VFdGkyV3ZtVWR4bXo3M3RiZW42TWI-WazK5YV6jVBngF9J_uaF9XMfvpmj3EBl5Wkzcr0n6R5-e9ukTLQa0fFq5GslcbkxN3WNxq2q6pgqxG0XaZYYHA";
        let bearer = format!("Bearer {}", ticket);
        headers.append(AUTHORIZATION, bearer.parse().unwrap());

        let actual = handler.parse_auth_header(&headers);
        assert!(
            actual.is_err(),
            "should throw an error if no active subscription found for user"
        );
    }

    #[test]
    fn should_successfully_show_user_as_authenticated_and_return_ticket_payload_wrapper() {
        let user = "0xc142bcf040AbF93703c03DaCf02c54B40dA0eDEb";
        let chain_id = 421613;
        let contract_address = "0x29f49a438c747e7Dd1bfe7926b03783E47f9447B"
            .parse::<Address>()
            .unwrap();
        let domain_separator = TicketVerificationDomain {
            contract: H160(contract_address.0),
            chain_id: U256::from(chain_id),
        };
        let mut subscription_map: HashMap<Address, UserSubscriptionWithSigners> = HashMap::new();
        subscription_map.insert(
            user.parse::<Address>().unwrap(),
            UserSubscriptionWithSigners {
                user: user.parse::<Address>().unwrap(),
                signers: vec![user.parse::<Address>().unwrap()],
                start: Utc::now(),
                end: Utc::now().add(Duration::days(30)),
                rate: 1000,
            },
        );
        let subscription_ptr = Ptr::new(subscription_map);
        let subscriptions = Eventual::from_value(subscription_ptr);
        let handler = AuthHandler::create(domain_separator, subscriptions);
        let mut headers = HeaderMap::new();
        let ticket = "o2RuYW1lTnRlc3RfYXBpX2tleV8xZnNpZ25lclTBQrzwQKv5NwPAPazwLFS0DaDt63FhbGxvd2VkX3N1YmdyYXBoc1gsM25YZkszUmJGcmo2bWhrR2RvS1Jvd0VFdGkyV3ZtVWR4bXo3M3RiZW42TWI-WazK5YV6jVBngF9J_uaF9XMfvpmj3EBl5Wkzcr0n6R5-e9ukTLQa0fFq5GslcbkxN3WNxq2q6pgqxG0XaZYYHA";
        let bearer = format!("Bearer {}", ticket);
        headers.append(AUTHORIZATION, bearer.parse().unwrap());

        let expected_ticket_payload = TicketPayload::from_ticket_base64(
            &TicketVerificationDomain {
                contract: H160(contract_address.0),
                chain_id: U256::from(chain_id),
            },
            ticket,
        )
        .unwrap()
        .0;

        let actual = handler.parse_auth_header(&headers);
        assert!(
            actual.is_ok(),
            "should return the auth user TicketPayloadWrapper"
        );
        let actual_res = actual.unwrap();
        let actual_ticket_payload = actual_res.clone().ticket_payload;
        assert_eq!(actual_ticket_payload.user, expected_ticket_payload.user);
        assert_eq!(actual_ticket_payload.signer, expected_ticket_payload.signer);

        let actual_active_subscription = actual_res.clone().active_subscription;
        assert_eq!(
            actual_active_subscription.user,
            user.parse::<Address>().unwrap()
        );
    }
}
