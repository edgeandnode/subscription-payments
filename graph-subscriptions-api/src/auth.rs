use std::collections::HashMap;

use anyhow::{anyhow, ensure, Ok, Result};
use axum::http::{header::AUTHORIZATION, HeaderMap};
use eventuals::{Eventual, Ptr};
use graph_subscriptions::{eip712::DomainSeparator, TicketPayload};
use thiserror::Error;
use toolshed::bytes::Address;

use crate::subscriptions_subgraph::Subscription;

#[derive(Clone, Debug)]
pub struct TicketPayloadWrapper {
    pub ticket_payload: TicketPayload,
    pub subscription: Subscription,
}

pub struct AuthHandler {
    pub subscriptions_domain_separator: DomainSeparator,
    pub subscriptions: Eventual<Ptr<HashMap<Address, Subscription>>>,
}

impl AuthHandler {
    pub fn create(
        subscriptions_domain_separator: DomainSeparator,
        subscriptions: Eventual<Ptr<HashMap<Address, Subscription>>>,
    ) -> &'static Self {
        Box::leak(Box::new(Self {
            subscriptions_domain_separator,
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
        let (payload, _) = TicketPayload::from_ticket_base64(
            raw_auth_header.as_bytes(),
            &self.subscriptions_domain_separator,
        )?;

        let user = Address(payload.user.unwrap_or(payload.signer).0);
        let subscription = self
            .subscriptions
            .value_immediate()
            .unwrap_or_default()
            .get(&user)
            .cloned()
            .ok_or_else(|| anyhow!("Subscription not found for user {}", user))?;
        let signer = Address(payload.signer.0);
        ensure!(
            (signer == user) || subscription.signers.contains(&signer),
            "Signer {} not authorized for user {}",
            signer,
            user,
        );

        Ok(TicketPayloadWrapper {
            ticket_payload: payload,
            subscription,
        })
    }
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Authorization Header not found on request, or is invalid")]
    Unauthorized,
}
