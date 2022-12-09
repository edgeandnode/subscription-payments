import {
  Subscribe as SubscribeEvent,
  Unsubscribe as UnsubscribeEvent,
} from '../generated/Subscriptions/Subscriptions';
import {Subscribe, Unsubscribe, Subscription} from '../generated/schema';
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

  let subscription = new Subscription(event.params.subscriber);
  subscription.subscriber = event.params.subscriber;
  subscription.startBlock = event.params.startBlock;
  subscription.endBlock = event.params.endBlock;
  subscription.pricePerBlock = event.params.pricePerBlock;
  subscription.save();
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

  store.remove('Subscription', event.params.subscriber.toHexString());
}
