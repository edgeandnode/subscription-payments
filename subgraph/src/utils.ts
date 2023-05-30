import {
  Address,
  BigInt,
  ByteArray,
  Bytes,
  crypto,
} from '@graphprotocol/graph-ts';

import {UserSubscription} from '../generated/schema';
import {Unsubscribe as UnsubscribeEvent} from '../generated/Subscriptions/Subscriptions';

import {BILLING_PERIOD_SECONDS_BIGINT} from './constants';

/**
 * Generate a keccak256 hex string of the user:authorizedSigner
 * @param subscriptionOwner address of the Subscription owner
 * @param authorizedSigner address of the user authorized to sign for the owner of the Subscription
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
 * Generate a keccak256 hex string of the userSubscriptionId:start
 * @param user the UserSubscription the BillingPeriod tracks against id
 * @param start the start timestamp of the BillingPeriod
 * @returns Bytes representation of a hex string concatenation of the `userSubscriptionId:start` to create a unique id
 */
export function buildBillingPeriodId(
  userSubscriptionId: Bytes,
  start: BigInt
): Bytes {
  let hash = crypto.keccak256(
    ByteArray.fromUTF8(
      `${userSubscriptionId.toHexString()}:${start.toString()}`
    )
  );
  return Bytes.fromByteArray(hash);
}

/**
 * A BillingPeriod is currently 30days, add this period to the input `start` to get the `end` timestamp for the BillingPeriod.
 * @param start unix-timestamp start of the BillingPeriod
 * @returns the unix-timestamp end of the BillingPeriod
 */
export function buildBillingPeriodEnd(start: BigInt): BigInt {
  return start.plus(BILLING_PERIOD_SECONDS_BIGINT);
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
    ByteArray.fromUTF8(
      `${user.toHexString()}:${eventType}:${blockTimestamp.toString()}`
    )
  );
  return Bytes.fromByteArray(hash);
}

/**
 * Calculate the unlocked tokens being returned to the User for cancelling their Subscription.
 */
export function calculateUnlockedTokens(
  sub: UserSubscription,
  event: UnsubscribeEvent
): BigInt {
  let correctedStart = event.block.timestamp;
  if (sub.start > event.block.timestamp) {
    correctedStart = sub.start;
  }
  let diff = sub.end.minus(correctedStart);
  if (diff < new BigInt(0)) {
    diff = new BigInt(0);
  }
  return diff.times(sub.rate);
}
