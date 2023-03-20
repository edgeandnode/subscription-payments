import {store, log} from '@graphprotocol/graph-ts';

import {
  Init as InitEvent,
  Subscribe as SubscribeEvent,
  Unsubscribe as UnsubscribeEvent,
  AuthorizedSignerAdded as AuthorizedSignerAddedEvent,
  AuthorizedSignerRemoved as AuthorizedSignerRemovedEvent,
} from '../generated/Subscriptions/Subscriptions';
import {
  ActiveSubscription,
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
import {buildAuthorizedSignerId, buildUserSubscriptionEventId} from './utils';

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

  let sub = ActiveSubscription.load(event.params.user);
  if (sub == null) {
    sub = new ActiveSubscription(event.params.user);
    sub.user = user.id;
    sub.start = event.params.start;
    sub.end = event.params.end;
    sub.rate = event.params.rate;
    sub.save();
    // Since ActiveSubscription record does not exist, the user is subscribing for the 1st time.
    // Create and store a UserSubscriptionCreatedEvent record.
    buildAndSaveUserSubscriptionCreatedEvent(user, sub, event);
    // If the user calls the subscribe function on the contract and the ActiveSubscription.end > block.timestamp,
    // then the contract will call the unsubscribe function, which emits the Unsubscribe event.
    // In the `handleUnsubscribe` fn below, we create a `UserSubscriptionCanceledEvent` record on the Unsubscribe event.
    // The contract then recreates the ActiveSubscription and emits a Subscribe event; which is handled by this function.
    // In this instance where an Unsubscribe is immediately followed by a Subscribe,
    // the user did not intend to "Cancel" their subscription, they meant to renew it.
    // As a result, find the created `UserSubscriptionCanceledEvent` record for the user and remove it from the store,
    // and create a `UserSubscriptionRenewalEvent` record.
    // NOTE:  preferably, we don't create the `UserSubscriptionCanceledEvent` record, but not sure how to ensure that.
    //        alternatively, we create the `UserSubscriptionCanceledEvent` record but don't remove it to track that it occurred.
    let canceledEventId = buildUserSubscriptionEventId(
      user.id,
      USER_SUBSCRIPTION_EVENT_TYPE__CANCELED,
      event.block.timestamp
    );
    let canceledEvent = UserSubscriptionCanceledEvent.load(canceledEventId);
    if (canceledEvent != null) {
      buildAndSaveUserSubscriptionRenewalEvent(user, sub, event);

      store.remove(
        'UserSubscriptionCanceledEvent',
        canceledEventId.toHexString()
      );
    }
  } else {
    // Check if the sub.rate is > than the event.params.rate value.
    // If this is true, then the user is upgrading their ActiveSubscription; create a UserSubscriptionUpgradeEvent record.
    if (event.params.rate > sub.rate) {
      buildAndSaveUserSubscriptionUpgradeEvent(user, sub, event);
    }
    // Check if the sub.rate is < than the event.params.rate value.
    // If this is true, then the user is downgrading their ActiveSubscription; create a UserSubscriptionDowngradeEvent record.
    if (event.params.rate < sub.rate) {
      buildAndSaveUserSubscriptionDowngradeEvent(user, sub, event);
    }
    sub.user = user.id;
    sub.start = event.params.start;
    sub.end = event.params.end;
    sub.rate = event.params.rate;
    sub.save();
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

  buildAndSaveUserSubscriptionCanceledEvent(user, event);

  store.remove('ActiveSubscription', event.params.user.toHexString());
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
  activeSubscription: ActiveSubscription,
  event: SubscribeEvent
): void {
  let id = buildUserSubscriptionEventId(
    user.id,
    USER_SUBSCRIPTION_EVENT_TYPE__CREATED,
    event.block.timestamp
  );
  let createdEvent = new UserSubscriptionCreatedEvent(id);
  createdEvent.user = user.id;
  createdEvent.blockNumber = event.block.number;
  createdEvent.blockTimestamp = event.block.timestamp;
  createdEvent.txHash = event.transaction.hash;
  createdEvent.activeSubscription = activeSubscription.id;
  createdEvent.eventType = USER_SUBSCRIPTION_EVENT_TYPE__CREATED;
  createdEvent.save();
}

function buildAndSaveUserSubscriptionCanceledEvent(
  user: User,
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
  canceledEvent.save();
}

function buildAndSaveUserSubscriptionRenewalEvent(
  user: User,
  activeSubscription: ActiveSubscription,
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
  renewalEvent.activeSubscription = activeSubscription.id;
  renewalEvent.eventType = USER_SUBSCRIPTION_EVENT_TYPE__RENEW;
  renewalEvent.save();
}

function buildAndSaveUserSubscriptionUpgradeEvent(
  user: User,
  activeSubscription: ActiveSubscription,
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
  upgradeEvent.activeSubscription = activeSubscription.id;
  upgradeEvent.eventType = USER_SUBSCRIPTION_EVENT_TYPE__UPGRADE;
  upgradeEvent.save();
}

function buildAndSaveUserSubscriptionDowngradeEvent(
  user: User,
  activeSubscription: ActiveSubscription,
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
  downgradeEvent.activeSubscription = activeSubscription.id;
  downgradeEvent.eventType = USER_SUBSCRIPTION_EVENT_TYPE__DOWNGRADE;
  downgradeEvent.save();
}
