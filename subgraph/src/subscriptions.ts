import {store} from '@graphprotocol/graph-ts';

import {
  Init as InitEvent,
  Extend as ExtendEvent,
  Subscribe as SubscribeEvent,
  Unsubscribe as UnsubscribeEvent,
  AuthorizedSignerAdded as AuthorizedSignerAddedEvent,
  AuthorizedSignerRemoved as AuthorizedSignerRemovedEvent,
} from '../generated/Subscriptions/Subscriptions';
import {
  ActiveSubscription,
  Init,
  Extend,
  Subscribe,
  Unsubscribe,
  AuthorizedSigner,
} from '../generated/schema';

import {loadOrCreateUser} from './entity-loader';
import {buildAuthorizedSignerId} from './utils';

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

  let sub = new ActiveSubscription(event.params.user);
  sub.user = user.id;
  sub.start = event.params.start;
  sub.end = event.params.end;
  sub.rate = event.params.rate;
  sub.save();
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

  store.remove('ActiveSubscription', event.params.user.toHexString());
}

export function handleExtend(event: ExtendEvent): void {
  let user = loadOrCreateUser(event.params.user);

  let entity = new Extend(
    event.transaction.hash.concatI32(event.logIndex.toI32())
  );
  entity.blockNumber = event.block.number;
  entity.blockTimestamp = event.block.timestamp;
  entity.transactionHash = event.transaction.hash;
  entity.user = user.id;
  entity.end = event.params.end;
  entity.save();

  let sub = ActiveSubscription.load(event.params.user)!;
  sub.end = event.params.end;
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
