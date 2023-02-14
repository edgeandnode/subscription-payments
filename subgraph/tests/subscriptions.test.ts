import {
  assert,
  describe,
  test,
  clearStore,
  beforeAll,
  afterAll,
} from 'matchstick-as/assembly/index';

import {Address, BigInt} from '@graphprotocol/graph-ts';

import {
  handleExtend,
  handleSubscribe,
  handleUnsubscribe,
  handleAuthorizedSignerAdded,
  handleAuthorizedSignerRemoved,
} from '../src/subscriptions';
import {buildAuthorizedSignerId} from '../src/utils';
import {
  createAuthorizedSignerAddedEvent,
  createAuthorizedSignerRemovedEvent,
  createExtendEvent,
  createSubscribeEvent,
  createUnsubscribeEvent,
} from './subscriptions-utils';

// Tests structure (matchstick-as >=0.5.0)
// https://thegraph.com/docs/en/developer/matchstick/#tests-structure-0-5-0

describe('Describe entity assertions', () => {
  const user = '0x0000000000000000000000000000000000000001';

  beforeAll(() => {
    let event = createSubscribeEvent(
      Address.fromString(user),
      BigInt.fromU32(2000),
      BigInt.fromU32(5000),
      BigInt.fromU32(10)
        .pow(18)
        .times(BigInt.fromU32(2))
    );
    handleSubscribe(event);
  });

  afterAll(() => {
    clearStore();
  });

  test('handle Subscribe', () => {
    assert.entityCount('Subscribe', 1);

    // 0xa16081f360e3847006db660bae1c6d1b2e17ec2a is the default address used in newMockEvent() function
    const id = '0xa16081f360e3847006db660bae1c6d1b2e17ec2a' + '01000000';
    assert.fieldEquals('Subscribe', id, 'user', user);
    assert.fieldEquals('Subscribe', id, 'start', '2000');
    assert.fieldEquals('Subscribe', id, 'end', '5000');
    assert.fieldEquals('Subscribe', id, 'rate', '2000000000000000000');

    assert.entityCount('User', 1);

    assert.entityCount('ActiveSubscription', 1);
    assert.fieldEquals('ActiveSubscription', user, 'user', user);
    assert.fieldEquals('ActiveSubscription', user, 'start', '2000');
    assert.fieldEquals('ActiveSubscription', user, 'end', '5000');
    assert.fieldEquals(
      'ActiveSubscription',
      user,
      'rate',
      '2000000000000000000'
    );
  });

  test('handle Unsubscribe', () => {
    let event = createUnsubscribeEvent(Address.fromString(user));
    handleUnsubscribe(event);

    assert.entityCount('Subscribe', 1);
    assert.entityCount('Unsubscribe', 1);
    assert.entityCount('ActiveSubscription', 0);
  });

  test('update Subscription', () => {
    let event = createSubscribeEvent(
      Address.fromString(user),
      BigInt.fromU32(3000),
      BigInt.fromU32(8000),
      BigInt.fromU32(10)
        .pow(18)
        .times(BigInt.fromU32(3))
    );
    event.logIndex = BigInt.fromU32(2);
    handleSubscribe(event);

    assert.entityCount('Subscribe', 2);
    assert.entityCount('Unsubscribe', 1);

    assert.entityCount('ActiveSubscription', 1);
    assert.fieldEquals('ActiveSubscription', user, 'user', user);
    assert.fieldEquals('ActiveSubscription', user, 'start', '3000');
    assert.fieldEquals('ActiveSubscription', user, 'end', '8000');
    assert.fieldEquals(
      'ActiveSubscription',
      user,
      'rate',
      '3000000000000000000'
    );
  });

  test('extend Subscription', () => {
    let event = createExtendEvent(
      Address.fromString(user),
      BigInt.fromU32(10000)
    );
    event.logIndex = BigInt.fromU32(3);
    handleExtend(event);

    assert.entityCount('Extend', 1);
    assert.entityCount('Subscribe', 2);
    assert.entityCount('Unsubscribe', 1);

    assert.entityCount('ActiveSubscription', 1);
    assert.fieldEquals('ActiveSubscription', user, 'user', user);
    assert.fieldEquals('ActiveSubscription', user, 'start', '3000');
    assert.fieldEquals('ActiveSubscription', user, 'end', '10000');
    assert.fieldEquals(
      'ActiveSubscription',
      user,
      'rate',
      '3000000000000000000'
    );
  });

  test('should be able to add an AuthorizedSigner entity for the ActiveSubscription. but must be unique', () => {
    const signer = '0x0000000000000000000000000000000000000002';
    let subscriptionOwner = Address.fromString(user);
    let authorizedSigner = Address.fromString(signer);
    let event1 = createAuthorizedSignerAddedEvent(
      subscriptionOwner,
      authorizedSigner
    );
    event1.logIndex = BigInt.fromU32(4);

    handleAuthorizedSignerAdded(event1);

    let authorizedSignerEntityId = buildAuthorizedSignerId(
      subscriptionOwner,
      authorizedSigner
    );
    assert.fieldEquals(
      'AuthorizedSigner',
      authorizedSignerEntityId.toHexString(),
      'user',
      user
    );
    assert.fieldEquals(
      'AuthorizedSigner',
      authorizedSignerEntityId.toHexString(),
      'signer',
      authorizedSigner.toHexString()
    );

    // should not readd the same authorized signer
    let event2 = createAuthorizedSignerAddedEvent(
      subscriptionOwner,
      authorizedSigner
    );
    event2.logIndex = BigInt.fromU32(5);

    handleAuthorizedSignerAdded(event2);

    assert.entityCount('AuthorizedSigner', 1);
  });

  test('should be able to remove an AuthorizedSigner entity', () => {
    assert.entityCount('AuthorizedSigner', 1);
    // create another AuthorizedSigner
    const signer2 = '0x0000000000000000000000000000000000000003';
    let subscriptionOwner = Address.fromString(user);
    let authorizedSigner2 = Address.fromString(signer2);
    let authorizedSignerEntity2Id = buildAuthorizedSignerId(
      subscriptionOwner,
      authorizedSigner2
    );
    let addEvent = createAuthorizedSignerAddedEvent(
      subscriptionOwner,
      authorizedSigner2
    );
    addEvent.logIndex = BigInt.fromU32(6);

    handleAuthorizedSignerAdded(addEvent);

    assert.entityCount('AuthorizedSigner', 2);
    assert.fieldEquals(
      'AuthorizedSigner',
      authorizedSignerEntity2Id.toHexString(),
      'user',
      user
    );
    assert.fieldEquals(
      'AuthorizedSigner',
      authorizedSignerEntity2Id.toHexString(),
      'signer',
      authorizedSigner2.toHexString()
    );

    // remove first AuthorizedSigner entity
    const signer1 = '0x0000000000000000000000000000000000000002';
    let authorizedSigner1 = Address.fromString(signer1);
    let authorizedSignerEntity1Id = buildAuthorizedSignerId(
      subscriptionOwner,
      authorizedSigner1
    );
    let removeEvent = createAuthorizedSignerRemovedEvent(
      subscriptionOwner,
      Address.fromString(signer1)
    );
    removeEvent.logIndex = BigInt.fromU32(7);

    handleAuthorizedSignerRemoved(removeEvent);

    assert.entityCount('AuthorizedSigner', 1);
    assert.notInStore(
      'AuthorizedSigner',
      authorizedSignerEntity1Id.toHexString()
    );
  });
});
