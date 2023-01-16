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
} from '../src/subscriptions';
import {
  createExtendEvent,
  createSubscribeEvent,
  createUnsubscribeEvent,
} from './subscriptions-utils';

// Tests structure (matchstick-as >=0.5.0)
// https://thegraph.com/docs/en/developer/matchstick/#tests-structure-0-5-0

describe('Describe entity assertions', () => {
  const subscriber = '0x0000000000000000000000000000000000000001';

  beforeAll(() => {
    let event = createSubscribeEvent(
      Address.fromString(subscriber),
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
    assert.fieldEquals('Subscribe', id, 'subscriber', subscriber);
    assert.fieldEquals('Subscribe', id, 'startBlock', '2000');
    assert.fieldEquals('Subscribe', id, 'endBlock', '5000');
    assert.fieldEquals('Subscribe', id, 'pricePerBlock', '2000000000000000000');

    assert.entityCount('ActiveSubscription', 1);
    assert.fieldEquals(
      'ActiveSubscription',
      subscriber,
      'subscriber',
      subscriber
    );
    assert.fieldEquals('ActiveSubscription', subscriber, 'startBlock', '2000');
    assert.fieldEquals('ActiveSubscription', subscriber, 'endBlock', '5000');
    assert.fieldEquals(
      'ActiveSubscription',
      subscriber,
      'pricePerBlock',
      '2000000000000000000'
    );
  });

  test('handle Unsubscribe', () => {
    let event = createUnsubscribeEvent(Address.fromString(subscriber));
    handleUnsubscribe(event);

    assert.entityCount('Subscribe', 1);
    assert.entityCount('Unsubscribe', 1);
    assert.entityCount('ActiveSubscription', 0);
  });

  test('update Subscription', () => {
    let event = createSubscribeEvent(
      Address.fromString(subscriber),
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
    assert.fieldEquals(
      'ActiveSubscription',
      subscriber,
      'subscriber',
      subscriber
    );
    assert.fieldEquals('ActiveSubscription', subscriber, 'startBlock', '3000');
    assert.fieldEquals('ActiveSubscription', subscriber, 'endBlock', '8000');
    assert.fieldEquals(
      'ActiveSubscription',
      subscriber,
      'pricePerBlock',
      '3000000000000000000'
    );
  });

  test('extend Subscription', () => {
    let event = createExtendEvent(
      Address.fromString(subscriber),
      BigInt.fromU32(10000)
    );
    event.logIndex = BigInt.fromU32(3);
    handleExtend(event);

    assert.entityCount('Extend', 1);
    assert.entityCount('Subscribe', 2);
    assert.entityCount('Unsubscribe', 1);

    assert.entityCount('ActiveSubscription', 1);
    assert.fieldEquals(
      'ActiveSubscription',
      subscriber,
      'subscriber',
      subscriber
    );
    assert.fieldEquals('ActiveSubscription', subscriber, 'startBlock', '3000');
    assert.fieldEquals('ActiveSubscription', subscriber, 'endBlock', '10000');
    assert.fieldEquals(
      'ActiveSubscription',
      subscriber,
      'pricePerBlock',
      '3000000000000000000'
    );
  });
});
