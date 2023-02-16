import {newMockEvent} from 'matchstick-as';
import {ethereum, Address, BigInt} from '@graphprotocol/graph-ts';
import {
  AuthorizedSignerAdded,
  AuthorizedSignerRemoved,
  Subscribe,
  Unsubscribe,
} from '../generated/Subscriptions/Subscriptions';

export function createSubscribeEvent(
  user: Address,
  epoch: BigInt,
  start: BigInt,
  end: BigInt,
  pricePerBlock: BigInt
): Subscribe {
  let subscribeEvent = changetype<Subscribe>(newMockEvent());

  subscribeEvent.parameters = new Array();

  subscribeEvent.parameters.push(
    new ethereum.EventParam('user', ethereum.Value.fromAddress(user))
  );
  subscribeEvent.parameters.push(
    new ethereum.EventParam('epoch', ethereum.Value.fromUnsignedBigInt(epoch))
  );
  subscribeEvent.parameters.push(
    new ethereum.EventParam('start', ethereum.Value.fromUnsignedBigInt(start))
  );
  subscribeEvent.parameters.push(
    new ethereum.EventParam('end', ethereum.Value.fromUnsignedBigInt(end))
  );
  subscribeEvent.parameters.push(
    new ethereum.EventParam(
      'pricePerBlock',
      ethereum.Value.fromUnsignedBigInt(pricePerBlock)
    )
  );

  return subscribeEvent;
}

export function createUnsubscribeEvent(
  user: Address,
  epoch: BigInt
): Unsubscribe {
  let unsubscribeEvent = changetype<Unsubscribe>(newMockEvent());

  unsubscribeEvent.parameters = new Array();

  unsubscribeEvent.parameters.push(
    new ethereum.EventParam('user', ethereum.Value.fromAddress(user))
  );
  unsubscribeEvent.parameters.push(
    new ethereum.EventParam('epoch', ethereum.Value.fromUnsignedBigInt(epoch))
  );

  return unsubscribeEvent;
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
