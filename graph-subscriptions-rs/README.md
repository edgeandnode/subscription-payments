# graph-subscription-rs

Utilities for working with the Graph Subscriptions contract

## Tickets

A ticket has 2 parts:
1. Payload
2. Signature

The signature is always the last 65 bytes of the ticket. The ticket should be Base64Url encoded when they are sent to gateways along with queries.

### Ticket Payload

The payload is a [CBOR](https://www.rfc-editor.org/rfc/rfc7049)-encoded map. Only 2 fields are required:
1. `id: array(8)`: Unique ticket identifier
2. `signer: array(20)`: Address associated with the secret key used to sign the ticket.

Other optional fields may be supported at the gateway operator's discretion. See `TicketPayload` for the fields supported by this library.

### Ticket Signature

The signature is where things get a bit fun. We use [EIP-712](https://eips.ethereum.org/EIPS/eip-712) for signing the payload fields.

We could have chosen to sign the cbor-encoded payload for simplicity. However, EIP-712 will often provide a nicer user experience "for free" since users will be able to see the structure of the message they're signing (via a MetaMask interface for example). We use CBOR when transmitting the payload because it results in a more compact encoding than the Ethereum ABI.

### Additional Notes

- If you want the option to easily revoke a set of tickets without modifying an active subscription, you can derive a key pair from the user's signing key and add the address of the derived public key as an authorized signer for the user. Then the tickets signed with the derived signing key can be revoked by making a contract call to remove that authorized signer for the user.
