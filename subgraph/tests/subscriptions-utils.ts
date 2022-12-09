import {newMockEvent} from 'matchstick-as';
import {ethereum, Address, BigInt} from '@graphprotocol/graph-ts';
import {Subscribe, Unsubscribe} from '../generated/Subscriptions/Subscriptions';

export function createSubscribeEvent(
  subscriber: Address,
  startBlock: BigInt,
  endBlock: BigInt,
  pricePerBlock: BigInt
): Subscribe {
  let subscribeEvent = changetype<Subscribe>(newMockEvent());

  subscribeEvent.parameters = new Array();

  subscribeEvent.parameters.push(
    new ethereum.EventParam(
      'subscriber',
      ethereum.Value.fromAddress(subscriber)
    )
  );
  subscribeEvent.parameters.push(
    new ethereum.EventParam(
      'startBlock',
      ethereum.Value.fromUnsignedBigInt(startBlock)
    )
  );
  subscribeEvent.parameters.push(
    new ethereum.EventParam(
      'endBlock',
      ethereum.Value.fromUnsignedBigInt(endBlock)
    )
  );
  subscribeEvent.parameters.push(
    new ethereum.EventParam(
      'pricePerBlock',
      ethereum.Value.fromUnsignedBigInt(pricePerBlock)
    )
  );

  return subscribeEvent;
}

export function createUnsubscribeEvent(subscriber: Address): Unsubscribe {
  let unsubscribeEvent = changetype<Unsubscribe>(newMockEvent());

  unsubscribeEvent.parameters = new Array();

  unsubscribeEvent.parameters.push(
    new ethereum.EventParam(
      'subscriber',
      ethereum.Value.fromAddress(subscriber)
    )
  );

  return unsubscribeEvent;
}
