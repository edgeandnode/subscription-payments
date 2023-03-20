import {
  Address,
  BigInt,
  ByteArray,
  Bytes,
  crypto,
} from '@graphprotocol/graph-ts';

/**
 * Generate a keccak256 hex string of the user:authorizedSigner
 * @param subscriptionOwner address of the ActiveSubscription owner
 * @param authorizedSigner address of the user authorized to sign for the owner of the ActiveSubscription
 * @returns Bytes representation of a hex string concatenation of the `user:authorizedSigner` to create a unique id
 */
export function buildAuthorizedSignerId(
  subscriptionOwner: Address,
  authorizedSigner: Address
): Bytes {
  let hash = crypto.keccak256(
    ByteArray.fromUTF8(`${subscriptionOwner}:${authorizedSigner}`)
  );
  return Bytes.fromByteArray(hash);
}

/**
 * Generate a keccak256 hex string of the user:eventType:blockTimestamp
 * @param user the user the event belongs to/performed the action
 * @param eventType the UseSubscriptionEventType enum value
 * @param blockTimestamp the block timestamp of the event
 * @returns Bytes representation of a hex string concatenation of the `user:eventType:blockTimestamp` to create a unique id
 */
export function buildUserSubscriptionEventId(
  user: Bytes,
  eventType: string,
  blockTimestamp: BigInt
): Bytes {
  let hash = crypto.keccak256(
    ByteArray.fromUTF8(`${user}:${eventType}:${blockTimestamp.toString()}`)
  );
  return Bytes.fromByteArray(hash);
}
