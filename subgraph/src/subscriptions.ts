import {
  Extend as ExtendEvent,
  Subscribe as SubscribeEvent,
  Unsubscribe as UnsubscribeEvent,
} from '../generated/Subscriptions/Subscriptions';
import {
  ActiveSubscription,
  Extend,
  Subscribe,
  Unsubscribe,
} from '../generated/schema';
import {store} from '@graphprotocol/graph-ts';

export function handleSubscribe(event: SubscribeEvent): void {
  let entity = new Subscribe(
    event.transaction.hash.concatI32(event.logIndex.toI32())
  );
  entity.blockNumber = event.block.number;
  entity.blockTimestamp = event.block.timestamp;
  entity.transactionHash = event.transaction.hash;
  entity.subscriber = event.params.subscriber;
  entity.startBlock = event.params.startBlock;
  entity.endBlock = event.params.endBlock;
  entity.pricePerBlock = event.params.pricePerBlock;
  entity.save();

  let sub = new ActiveSubscription(event.params.subscriber);
  sub.subscriber = event.params.subscriber;
  sub.startBlock = event.params.startBlock;
  sub.endBlock = event.params.endBlock;
  sub.pricePerBlock = event.params.pricePerBlock;
  sub.save();
}

export function handleUnsubscribe(event: UnsubscribeEvent): void {
  let entity = new Unsubscribe(
    event.transaction.hash.concatI32(event.logIndex.toI32())
  );
  entity.blockNumber = event.block.number;
  entity.blockTimestamp = event.block.timestamp;
  entity.transactionHash = event.transaction.hash;
  entity.subscriber = event.params.subscriber;
  entity.save();

  store.remove('ActiveSubscription', event.params.subscriber.toHexString());
}

export function handleExtend(event: ExtendEvent): void {
  let entity = new Extend(
    event.transaction.hash.concatI32(event.logIndex.toI32())
  );
  entity.blockNumber = event.block.number;
  entity.blockTimestamp = event.block.timestamp;
  entity.transactionHash = event.transaction.hash;
  entity.subscriber = event.params.subscriber;
  entity.endBlock = event.params.endBlock;
  entity.save();

  let sub = ActiveSubscription.load(event.params.subscriber)!;
  sub.endBlock = event.params.endBlock;
  sub.save();
}
