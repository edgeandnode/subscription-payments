import {BigInt, store} from '@graphprotocol/graph-ts';

import {
  Init as InitEvent,
  Subscribe as SubscribeEvent,
  Unsubscribe as UnsubscribeEvent,
  AuthorizedSignerAdded as AuthorizedSignerAddedEvent,
  AuthorizedSignerRemoved as AuthorizedSignerRemovedEvent,
} from '../generated/Subscriptions/Subscriptions';
import {
  Subscription,
  Init,
  Subscribe,
  Unsubscribe,
  AuthorizedSigner,
  UserSubscriptionCanceledEvent,
  UserSubscriptionCreatedEvent,
  UserSubscriptionDowngradeEvent,
  UserSubscriptionRenewalEvent,
  UserSubscriptionUpgradeEvent,
  User,
} from '../generated/schema';

import {
  USER_SUBSCRIPTION_EVENT_TYPE__CANCELED,
  USER_SUBSCRIPTION_EVENT_TYPE__CREATED,
  USER_SUBSCRIPTION_EVENT_TYPE__DOWNGRADE,
  USER_SUBSCRIPTION_EVENT_TYPE__RENEW,
  USER_SUBSCRIPTION_EVENT_TYPE__UPGRADE,
} from './constants';
import {loadOrCreateUser} from './entity-loader';
import {
  buildAuthorizedSignerId,
  buildUserSubscriptionEventId,
  calculateUnlockedTokens,
} from './utils';

export function handleInit(event: InitEvent): void {
  let entity = new Init(
    event.transaction.hash.concatI32(event.logIndex.toI32())
  );
  entity.blockNumber = event.block.number;
  entity.blockTimestamp = event.block.timestamp;
  entity.transactionHash = event.transaction.hash;
  entity.token = event.params.token;
  entity.save();
}

export function handleSubscribe(event: SubscribeEvent): void {
  let entity = new Subscribe(
    event.transaction.hash.concatI32(event.logIndex.toI32())
  );
  entity.blockNumber = event.block.number;
  entity.blockTimestamp = event.block.timestamp;
  entity.transactionHash = event.transaction.hash;
  entity.user = event.params.user;
  entity.start = event.params.start;
  entity.end = event.params.end;
  entity.rate = event.params.rate;
  entity.save();

  let user = loadOrCreateUser(event.params.user);

  let sub = Subscription.load(event.params.user);
  if (sub == null) {
    sub = new Subscription(event.params.user);
    sub.user = user.id;
    sub.start = event.params.start;
    sub.end = event.params.end;
    sub.rate = event.params.rate;
    sub.cancelled = false;
    sub.save();
    // Since Subscription record does not exist, the user is subscribing for the 1st time.
    // Create and store a UserSubscriptionCreatedEvent record.
    buildAndSaveUserSubscriptionCreatedEvent(user, sub, event);
    return;
  }

  // Check if the sub.rate is > than the event.params.rate value.
  // If this is true, then the user is upgrading their Subscription; create a UserSubscriptionUpgradeEvent record.
  if (event.params.rate > sub.rate) {
    buildAndSaveUserSubscriptionUpgradeEvent(user, sub, event);
  }
  // Check if the sub.rate is < than the event.params.rate value.
  // If this is true, then the user is downgrading their Subscription; create a UserSubscriptionDowngradeEvent record.
  else if (event.params.rate < sub.rate) {
    buildAndSaveUserSubscriptionDowngradeEvent(user, sub, event);
  } else {
    buildAndSaveUserSubscriptionRenewalEvent(user, sub, event);
  }

  sub.user = user.id;
  sub.start = event.params.start;
  sub.end = event.params.end;
  sub.rate = event.params.rate;
  sub.cancelled = false;
  sub.save();

  // If a CanceledEvent was created in the same block, we remove it.
  const cancelEvent = UserSubscriptionCanceledEvent.load(
    buildUserSubscriptionEventId(
      user.id,
      USER_SUBSCRIPTION_EVENT_TYPE__CANCELED,
      event.block.timestamp
    )
  );

  if (cancelEvent != null) {
    store.remove('UserSubscriptionCanceledEvent', cancelEvent.id.toHexString());
    user.eventCount = user.eventCount - 1;
    user.save();
  }
}

export function handleUnsubscribe(event: UnsubscribeEvent): void {
  let user = loadOrCreateUser(event.params.user);

  let entity = new Unsubscribe(
    event.transaction.hash.concatI32(event.logIndex.toI32())
  );
  entity.blockNumber = event.block.number;
  entity.blockTimestamp = event.block.timestamp;
  entity.transactionHash = event.transaction.hash;
  entity.user = user.id;
  entity.save();

  let sub = Subscription.load(event.params.user);
  if (sub == null) return;

  // To handle an edge-case where the Subscribe/Unsubscribe events aren't received by the subgraph mapping in the same order they are emitted,
  // if a `UserSubscriptionCreatedEvent` exists in the same timestamp, don't create the `UserSubscriptionCanceledEvent` record
  let subscribeEvent = Subscribe.load(event.transaction.hash);
  if (subscribeEvent != null) return;

  buildAndSaveUserSubscriptionCanceledEvent(user, sub, event);

  sub.cancelled = true;
  sub.save();
}

export function handleAuthorizedSignerAdded(
  event: AuthorizedSignerAddedEvent
): void {
  let user = loadOrCreateUser(event.params.subscriptionOwner);

  let subscriptionOwner = event.params.subscriptionOwner;
  let authorizedSigner = event.params.authorizedSigner;
  let id = buildAuthorizedSignerId(subscriptionOwner, authorizedSigner);
  // validate an AuthorizedSigner entity with the id doesn't already exist
  let signer = AuthorizedSigner.load(id);
  if (signer != null) {
    return;
  }
  signer = new AuthorizedSigner(id);
  signer.user = user.id;
  signer.signer = authorizedSigner;
  signer.save();
}

export function handleAuthorizedSignerRemoved(
  event: AuthorizedSignerRemovedEvent
): void {
  let id = buildAuthorizedSignerId(
    event.params.subscriptionOwner,
    event.params.authorizedSigner
  );
  store.remove('AuthorizedSigner', id.toHexString());
}

function buildAndSaveUserSubscriptionCreatedEvent(
  user: User,
  sub: Subscription,
  event: SubscribeEvent
): void {
  let id = buildUserSubscriptionEventId(
    user.id,
    USER_SUBSCRIPTION_EVENT_TYPE__CREATED,
    event.block.timestamp
  );
  if (UserSubscriptionCreatedEvent.load(id) != null) {
    return;
  }
  let createdEvent = new UserSubscriptionCreatedEvent(id);
  createdEvent.user = user.id;
  createdEvent.blockNumber = event.block.number;
  createdEvent.blockTimestamp = event.block.timestamp;
  createdEvent.txHash = event.transaction.hash;
  createdEvent.currentSubscriptionStart = sub.start;
  createdEvent.currentSubscriptionEnd = sub.end;
  createdEvent.currentSubscriptionRate = sub.rate;
  createdEvent.eventType = USER_SUBSCRIPTION_EVENT_TYPE__CREATED;
  createdEvent.save();

  incrementUserEventCount(user);
}

function buildAndSaveUserSubscriptionCanceledEvent(
  user: User,
  sub: Subscription,
  event: UnsubscribeEvent
): void {
  let id = buildUserSubscriptionEventId(
    user.id,
    USER_SUBSCRIPTION_EVENT_TYPE__CANCELED,
    event.block.timestamp
  );
  let canceledEvent = new UserSubscriptionCanceledEvent(id);
  canceledEvent.user = user.id;
  canceledEvent.blockNumber = event.block.number;
  canceledEvent.blockTimestamp = event.block.timestamp;
  canceledEvent.txHash = event.transaction.hash;
  canceledEvent.eventType = USER_SUBSCRIPTION_EVENT_TYPE__CANCELED;
  canceledEvent.tokensReturned = calculateUnlockedTokens(sub, event);
  canceledEvent.save();

  incrementUserEventCount(user);
}

function buildAndSaveUserSubscriptionRenewalEvent(
  user: User,
  sub: Subscription,
  event: SubscribeEvent
): void {
  let id = buildUserSubscriptionEventId(
    user.id,
    USER_SUBSCRIPTION_EVENT_TYPE__RENEW,
    event.block.timestamp
  );
  let renewalEvent = new UserSubscriptionRenewalEvent(id);
  renewalEvent.user = user.id;
  renewalEvent.blockNumber = event.block.number;
  renewalEvent.blockTimestamp = event.block.timestamp;
  renewalEvent.txHash = event.transaction.hash;
  renewalEvent.currentSubscriptionStart = sub.start;
  renewalEvent.currentSubscriptionEnd = sub.end;
  renewalEvent.currentSubscriptionRate = sub.rate;
  renewalEvent.eventType = USER_SUBSCRIPTION_EVENT_TYPE__RENEW;
  renewalEvent.save();

  incrementUserEventCount(user);
}

function buildAndSaveUserSubscriptionUpgradeEvent(
  user: User,
  sub: Subscription,
  event: SubscribeEvent
): void {
  let id = buildUserSubscriptionEventId(
    user.id,
    USER_SUBSCRIPTION_EVENT_TYPE__UPGRADE,
    event.block.timestamp
  );
  let upgradeEvent = new UserSubscriptionUpgradeEvent(id);
  upgradeEvent.user = user.id;
  upgradeEvent.blockNumber = event.block.number;
  upgradeEvent.blockTimestamp = event.block.timestamp;
  upgradeEvent.txHash = event.transaction.hash;
  upgradeEvent.eventType = USER_SUBSCRIPTION_EVENT_TYPE__UPGRADE;
  upgradeEvent.currentSubscriptionStart = sub.start;
  upgradeEvent.currentSubscriptionEnd = sub.end;
  upgradeEvent.currentSubscriptionRate = sub.rate;
  upgradeEvent.previousSubscriptionStart = sub.start;
  upgradeEvent.previousSubscriptionEnd = sub.end;
  upgradeEvent.previousSubscriptionRate = sub.rate;
  upgradeEvent.save();

  incrementUserEventCount(user);
}

function buildAndSaveUserSubscriptionDowngradeEvent(
  user: User,
  sub: Subscription,
  event: SubscribeEvent
): void {
  let id = buildUserSubscriptionEventId(
    user.id,
    USER_SUBSCRIPTION_EVENT_TYPE__DOWNGRADE,
    event.block.timestamp
  );
  let downgradeEvent = new UserSubscriptionDowngradeEvent(id);
  downgradeEvent.user = user.id;
  downgradeEvent.blockNumber = event.block.number;
  downgradeEvent.blockTimestamp = event.block.timestamp;
  downgradeEvent.txHash = event.transaction.hash;
  downgradeEvent.eventType = USER_SUBSCRIPTION_EVENT_TYPE__DOWNGRADE;
  downgradeEvent.currentSubscriptionStart = sub.start;
  downgradeEvent.currentSubscriptionEnd = sub.end;
  downgradeEvent.currentSubscriptionRate = sub.rate;
  downgradeEvent.previousSubscriptionStart = sub.start;
  downgradeEvent.previousSubscriptionEnd = sub.end;
  downgradeEvent.previousSubscriptionRate = sub.rate;
  downgradeEvent.save();

  incrementUserEventCount(user);
}

function incrementUserEventCount(user: User): void {
  user.eventCount = user.eventCount + 1;
  user.save();
}
