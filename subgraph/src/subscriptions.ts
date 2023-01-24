import {
  Init as InitEvent,
  Subscribe as SubscribeEvent,
  Unsubscribe as UnsubscribeEvent,
} from '../generated/Subscriptions/Subscriptions';
import {
  ActiveSubscription,
  Init,
  Subscribe,
  Unsubscribe,
} from '../generated/schema';
import {store} from '@graphprotocol/graph-ts';

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

  let sub = new ActiveSubscription(event.params.user);
  sub.user = event.params.user;
  sub.start = event.params.start;
  sub.end = event.params.end;
  sub.rate = event.params.rate;
  sub.save();
}

export function handleUnsubscribe(event: UnsubscribeEvent): void {
  let entity = new Unsubscribe(
    event.transaction.hash.concatI32(event.logIndex.toI32())
  );
  entity.blockNumber = event.block.number;
  entity.blockTimestamp = event.block.timestamp;
  entity.transactionHash = event.transaction.hash;
  entity.user = event.params.user;
  entity.save();

  store.remove('ActiveSubscription', event.params.user.toHexString());
}

// export function handleExtend(event: ExtendEvent): void {
//   let entity = new Extend(
//     event.transaction.hash.concatI32(event.logIndex.toI32())
//   );
//   entity.blockNumber = event.block.number;
//   entity.blockTimestamp = event.block.timestamp;
//   entity.transactionHash = event.transaction.hash;
//   entity.user = event.params.user;
//   entity.end = event.params.end;
//   entity.save();

//   let sub = ActiveSubscription.load(event.params.user)!;
//   sub.end = event.params.end;
//   sub.save();
// }
