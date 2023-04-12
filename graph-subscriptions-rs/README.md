# graph-subscription-rs

Utilities for working with the Graph Subscriptions contract

## Tickets

A ticket has 2 parts:
1. Payload
2. Signature

The signature is always the last 65 bytes of the ticket. The ticket should be Base64Url encoded when they are sent to gateways along with queries.

### Ticket Payload

The payload is a [CBOR](https://www.rfc-editor.org/rfc/rfc7049)-encoded map. The following fields must be supported:
1. `chain_id: U256`: EIP-155 chain id.
2. `contract: array(20)`: Address of the subscriptions contract.
3. `signer: array(20)`: Address associated with the secret key used to sign the ticket.
4. `user: optional array(20)`: Required to when the authorized `signer` is not the `user` associated with a subscription. When omitted, the `signer` is implied to be equal to the `user`.

Other optional fields may be supported at the gateway operator's discretion. See `TicketPayload` for the fields supported by this library.

### Ticket Signature

Signing and verification of tickets uses an Ethereum signed message ([EIP-191](https://eips.ethereum.org/EIPS/eip-191), `personal_sign`) constructed from the payload content.
- The message must be UTF-8 encoded.
- Fields must be ordered lexicographically by field name.
- Each field must be immediately followed by an ASCII LF character (`0x0a`).
- Each field name and value must be separated by `": "`.
- Any byte string value must be formatted as `0x` followed by its hex-encoded bytes.

See `TicketPayload::verification_message` for implementation.

We could have chosen to sign the CBOR-encoded payload for simplicity. However, EIP-191 messages will often provide a nicer user experience "for free" since users will be able to see the content of the message they're signing (via a MetaMask interface for example). We use CBOR when transmitting the payload because it results in a more compact encoding.

EIP-712 was previously used. However, it wasn't a perfect fit for similar reasons listed in the spec for [Sign-In with Ethereum](https://eips.ethereum.org/EIPS/eip-4361#technical-decisions).

### Additional Notes

- If you want the option to easily revoke a set of tickets without modifying an active subscription, you can derive a key pair from the user's signing key and add the address of the derived public key as an authorized signer for the user. Then the tickets signed with the derived signing key can be revoked by making a contract call to remove that authorized signer for the user.
