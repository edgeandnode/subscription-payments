import {newMockEvent} from 'matchstick-as';
import {ethereum, Address, BigInt} from '@graphprotocol/graph-ts';
import {
  AuthorizedSignerAdded,
  AuthorizedSignerRemoved,
  Extend,
  Subscribe,
  Unsubscribe,
} from '../generated/Subscriptions/Subscriptions';

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

export function createExtendEvent(
  subscriber: Address,
  endBlock: BigInt
): Extend {
  let extendEvent = changetype<Extend>(newMockEvent());

  extendEvent.parameters = new Array();

  extendEvent.parameters.push(
    new ethereum.EventParam(
      'subscriber',
      ethereum.Value.fromAddress(subscriber)
    )
  );
  extendEvent.parameters.push(
    new ethereum.EventParam(
      'endBlock',
      ethereum.Value.fromUnsignedBigInt(endBlock)
    )
  );

  return extendEvent;
}

export function createAuthorizedSignerAddedEvent(
  subscriptionOwner: Address,
  authorizedSigner: Address
): AuthorizedSignerAdded {
  let authorizedSignerAddedEvent = changetype<AuthorizedSignerAdded>(
    newMockEvent()
  );

  authorizedSignerAddedEvent.parameters = new Array();

  authorizedSignerAddedEvent.parameters.push(
    new ethereum.EventParam(
      'subscriptionOwner',
      ethereum.Value.fromAddress(subscriptionOwner)
    )
  );
  authorizedSignerAddedEvent.parameters.push(
    new ethereum.EventParam(
      'authorizedSigner',
      ethereum.Value.fromAddress(authorizedSigner)
    )
  );

  return authorizedSignerAddedEvent;
}

export function createAuthorizedSignerRemovedEvent(
  subscriptionOwner: Address,
  authorizedSigner: Address
): AuthorizedSignerRemoved {
  let authorizedSignerRemovedEvent = changetype<AuthorizedSignerRemoved>(
    newMockEvent()
  );

  authorizedSignerRemovedEvent.parameters = new Array();

  authorizedSignerRemovedEvent.parameters.push(
    new ethereum.EventParam(
      'subscriptionOwner',
      ethereum.Value.fromAddress(subscriptionOwner)
    )
  );
  authorizedSignerRemovedEvent.parameters.push(
    new ethereum.EventParam(
      'authorizedSigner',
      ethereum.Value.fromAddress(authorizedSigner)
    )
  );

  return authorizedSignerRemovedEvent;
}
