use anyhow::{ensure, Ok, Result};
use async_graphql::ErrorExtensions;
use axum::http::{header::AUTHORIZATION, HeaderMap};
use graph_subscriptions::{eip712::DomainSeparator, TicketPayload};
use thiserror::Error;

pub struct AuthHandler {
    pub subscriptions_domain_separator: DomainSeparator,
}

impl AuthHandler {
    pub fn create(subscriptions_domain_separator: DomainSeparator) -> &'static Self {
        Box::leak(Box::new(Self {
            subscriptions_domain_separator,
        }))
    }

    pub fn parse_auth_header(&self, headers: &HeaderMap) -> Result<TicketPayload> {
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

        Ok(payload)
    }
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Authorization Header not found on request, or is invalid")]
    Unauthorized,
}
impl ErrorExtensions for AuthError {
    fn extend(&self) -> async_graphql::Error {
        async_graphql::Error::new(format!("{}", self)).extend_with(|_, e| match self {
            AuthError::Unauthorized => e.set("code", "UNAUTHORIZED"),
        })
    }
}
