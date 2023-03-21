import {
  assert,
  describe,
  test,
  clearStore,
  beforeAll,
  afterAll,
} from 'matchstick-as/assembly/index';

import {Address, BigInt, Bytes} from '@graphprotocol/graph-ts';

import {ActiveSubscription} from '../generated/schema';

import {
  USER_SUBSCRIPTION_EVENT_TYPE__CANCELED,
  USER_SUBSCRIPTION_EVENT_TYPE__CREATED,
  USER_SUBSCRIPTION_EVENT_TYPE__DOWNGRADE,
  USER_SUBSCRIPTION_EVENT_TYPE__UPGRADE,
} from '../src/constants';
import {
  handleSubscribe,
  handleUnsubscribe,
  handleAuthorizedSignerAdded,
  handleAuthorizedSignerRemoved,
} from '../src/subscriptions';
import {
  buildAuthorizedSignerId,
  buildUserSubscriptionEventId,
  calculateUnlockedTokens,
} from '../src/utils';

import {
  createAuthorizedSignerAddedEvent,
  createAuthorizedSignerRemovedEvent,
  createSubscribeEvent,
  createUnsubscribeEvent,
} from './subscriptions-utils';

// Tests structure (matchstick-as >=0.5.0)
// https://thegraph.com/docs/en/developer/matchstick/#tests-structure-0-5-0

describe('Describe entity assertions', () => {
  const user = '0x0000000000000000000000000000000000000001';
  const timestamp = 1; // default block timestamp

  beforeAll(() => {
    let event = createSubscribeEvent(
      Address.fromString(user),
      BigInt.fromU32(0),
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
    assert.fieldEquals('User', user, 'eventCount', '1'); // 1 UserSubscriptionCreatedEvent

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

    // validate UserSubscriptionCreatedEvent record created
    assert.entityCount('UserSubscriptionCreatedEvent', 1);
    const createdEventId = buildUserSubscriptionEventId(
      Bytes.fromHexString(user),
      USER_SUBSCRIPTION_EVENT_TYPE__CREATED,
      BigInt.fromI32(timestamp)
    );
    assert.fieldEquals(
      'UserSubscriptionCreatedEvent',
      createdEventId.toHex(),
      'eventType',
      USER_SUBSCRIPTION_EVENT_TYPE__CREATED
    );
    assert.fieldEquals(
      'UserSubscriptionCreatedEvent',
      createdEventId.toHex(),
      'currentSubscriptionStart',
      '2000'
    );
    assert.fieldEquals(
      'UserSubscriptionCreatedEvent',
      createdEventId.toHex(),
      'currentSubscriptionEnd',
      '5000'
    );
    assert.fieldEquals(
      'UserSubscriptionCreatedEvent',
      createdEventId.toHex(),
      'currentSubscriptionRate',
      '2000000000000000000'
    );
  });

  test('handle Unsubscribe', () => {
    // build ActiveSubscription that is being removed (this is the ActiveSub that gets build in the beforeAll hook)
    let sub = new ActiveSubscription(Address.fromString(user));
    sub.start = BigInt.fromU32(2000);
    sub.end = BigInt.fromU32(5000);
    sub.rate = BigInt.fromU32(10)
      .pow(18)
      .times(BigInt.fromU32(2));

    let event = createUnsubscribeEvent(
      Address.fromString(user),
      BigInt.fromU32(0)
    );
    handleUnsubscribe(event);

    assert.entityCount('Subscribe', 1);
    assert.entityCount('Unsubscribe', 1);
    assert.entityCount('ActiveSubscription', 0);
    assert.entityCount('UserSubscriptionCanceledEvent', 1);
    const canceledEventId = buildUserSubscriptionEventId(
      Bytes.fromHexString(user),
      USER_SUBSCRIPTION_EVENT_TYPE__CANCELED,
      BigInt.fromI32(timestamp)
    );
    assert.fieldEquals(
      'UserSubscriptionCanceledEvent',
      canceledEventId.toHex(),
      'eventType',
      USER_SUBSCRIPTION_EVENT_TYPE__CANCELED
    );
    let tokensReturned = calculateUnlockedTokens(sub, event);
    assert.fieldEquals(
      'UserSubscriptionCanceledEvent',
      canceledEventId.toHex(),
      'tokensReturned',
      tokensReturned.toString()
    );

    assert.fieldEquals('User', user, 'eventCount', '2'); // 1 UserSubscriptionCreatedEvent, 1 UserSubscriptionCanceledEvent
  });

  test('update Subscription', () => {
    let event = createSubscribeEvent(
      Address.fromString(user),
      BigInt.fromU32(0),
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
    // validate only 1 UserSubscriptionCreatedEvent record created
    assert.entityCount('UserSubscriptionCreatedEvent', 1);
    // validate that a UserSubscriptionRenewalEvent is created as the subscription was extended
    assert.entityCount('UserSubscriptionRenewalEvent', 1);
    // validate that the UserSubscriptionCanceledEvent is removed
    assert.entityCount('UserSubscriptionCanceledEvent', 0);
    assert.fieldEquals('User', user, 'eventCount', '2'); // 1 UserSubscriptionCreatedEvent, 1 UserSubscriptionRenewalEvent
  });

  test('upgrade Subscription', () => {
    let event = createSubscribeEvent(
      Address.fromString(user),
      BigInt.fromU32(0),
      BigInt.fromU32(3000),
      BigInt.fromU32(8000),
      BigInt.fromU32(10)
        .pow(18)
        .times(BigInt.fromU32(5))
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
      '5000000000000000000'
    );
    // validate only 1 UserSubscriptionCreatedEvent record created
    assert.entityCount('UserSubscriptionCreatedEvent', 1);
    // validate that a UserSubscriptionUpgradeEvent is created as the subscription was rate was increased
    assert.entityCount('UserSubscriptionUpgradeEvent', 1);
    let upgradeEventId = buildUserSubscriptionEventId(
      Bytes.fromHexString(user),
      USER_SUBSCRIPTION_EVENT_TYPE__UPGRADE,
      BigInt.fromI32(timestamp)
    );
    assert.fieldEquals(
      'UserSubscriptionUpgradeEvent',
      upgradeEventId.toHex(),
      'previousSubscriptionStart',
      '3000' // value from `update Subscription` test above before this event is received
    );
    assert.fieldEquals(
      'UserSubscriptionUpgradeEvent',
      upgradeEventId.toHex(),
      'previousSubscriptionEnd',
      '8000' // value from `update Subscription` test above before this event is received
    );
    assert.fieldEquals(
      'UserSubscriptionUpgradeEvent',
      upgradeEventId.toHex(),
      'previousSubscriptionRate',
      '3000000000000000000' // value from `update Subscription` test above before this event is received
    );

    assert.fieldEquals('User', user, 'eventCount', '3'); // 1 UserSubscriptionCreatedEvent, 1 UserSubscriptionRenewalEvent, 1 UserSubscriptionUpgradeEvent
  });

  test('downgrade Subscription', () => {
    let event = createSubscribeEvent(
      Address.fromString(user),
      BigInt.fromU32(0),
      BigInt.fromU32(3000),
      BigInt.fromU32(8000),
      BigInt.fromU32(10)
        .pow(18)
        .times(BigInt.fromU32(2))
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
      '2000000000000000000'
    );
    // validate only 1 UserSubscriptionCreatedEvent record created
    assert.entityCount('UserSubscriptionCreatedEvent', 1);
    // validate that a UserSubscriptionDowngradeEvent is created as the subscription was rate was increased
    assert.entityCount('UserSubscriptionDowngradeEvent', 1);
    let downgradeEventId = buildUserSubscriptionEventId(
      Bytes.fromHexString(user),
      USER_SUBSCRIPTION_EVENT_TYPE__DOWNGRADE,
      BigInt.fromI32(timestamp)
    );
    assert.fieldEquals(
      'UserSubscriptionDowngradeEvent',
      downgradeEventId.toHex(),
      'previousSubscriptionStart',
      '3000' // value from `upgrade Subscription` test above before this event is received
    );
    assert.fieldEquals(
      'UserSubscriptionDowngradeEvent',
      downgradeEventId.toHex(),
      'previousSubscriptionEnd',
      '8000' // value from `upgrade Subscription` test above before this event is received
    );
    assert.fieldEquals(
      'UserSubscriptionDowngradeEvent',
      downgradeEventId.toHex(),
      'previousSubscriptionRate',
      '5000000000000000000' // value from `upgrade Subscription` test above before this event is received
    );
    assert.fieldEquals('User', user, 'eventCount', '4'); // 1 UserSubscriptionCreatedEvent, 1 UserSubscriptionRenewalEvent, 1 UserSubscriptionUpgradeEvent, 1 UserSubscriptionDowngradeEvent
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
