// THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.

import {
  ethereum,
  JSONValue,
  TypedMap,
  Entity,
  Bytes,
  Address,
  BigInt
} from "@graphprotocol/graph-ts";

export class AuthorizedSignerAdded extends ethereum.Event {
  get params(): AuthorizedSignerAdded__Params {
    return new AuthorizedSignerAdded__Params(this);
  }
}

export class AuthorizedSignerAdded__Params {
  _event: AuthorizedSignerAdded;

  constructor(event: AuthorizedSignerAdded) {
    this._event = event;
  }

  get subscriptionOwner(): Address {
    return this._event.parameters[0].value.toAddress();
  }

  get authorizedSigner(): Address {
    return this._event.parameters[1].value.toAddress();
  }
}

export class AuthorizedSignerRemoved extends ethereum.Event {
  get params(): AuthorizedSignerRemoved__Params {
    return new AuthorizedSignerRemoved__Params(this);
  }
}

export class AuthorizedSignerRemoved__Params {
  _event: AuthorizedSignerRemoved;

  constructor(event: AuthorizedSignerRemoved) {
    this._event = event;
  }

  get subscriptionOwner(): Address {
    return this._event.parameters[0].value.toAddress();
  }

  get authorizedSigner(): Address {
    return this._event.parameters[1].value.toAddress();
  }
}

export class Extend extends ethereum.Event {
  get params(): Extend__Params {
    return new Extend__Params(this);
  }
}

export class Extend__Params {
  _event: Extend;

  constructor(event: Extend) {
    this._event = event;
  }

  get user(): Address {
    return this._event.parameters[0].value.toAddress();
  }

  get end(): BigInt {
    return this._event.parameters[1].value.toBigInt();
  }
}

export class Init extends ethereum.Event {
  get params(): Init__Params {
    return new Init__Params(this);
  }
}

export class Init__Params {
  _event: Init;

  constructor(event: Init) {
    this._event = event;
  }

  get token(): Address {
    return this._event.parameters[0].value.toAddress();
  }
}

export class OwnershipTransferred extends ethereum.Event {
  get params(): OwnershipTransferred__Params {
    return new OwnershipTransferred__Params(this);
  }
}

export class OwnershipTransferred__Params {
  _event: OwnershipTransferred;

  constructor(event: OwnershipTransferred) {
    this._event = event;
  }

  get previousOwner(): Address {
    return this._event.parameters[0].value.toAddress();
  }

  get newOwner(): Address {
    return this._event.parameters[1].value.toAddress();
  }
}

export class PendingSubscriptionCreated extends ethereum.Event {
  get params(): PendingSubscriptionCreated__Params {
    return new PendingSubscriptionCreated__Params(this);
  }
}

export class PendingSubscriptionCreated__Params {
  _event: PendingSubscriptionCreated;

  constructor(event: PendingSubscriptionCreated) {
    this._event = event;
  }

  get user(): Address {
    return this._event.parameters[0].value.toAddress();
  }

  get epoch(): BigInt {
    return this._event.parameters[1].value.toBigInt();
  }

  get start(): BigInt {
    return this._event.parameters[2].value.toBigInt();
  }

  get end(): BigInt {
    return this._event.parameters[3].value.toBigInt();
  }

  get rate(): BigInt {
    return this._event.parameters[4].value.toBigInt();
  }
}

export class Subscribe extends ethereum.Event {
  get params(): Subscribe__Params {
    return new Subscribe__Params(this);
  }
}

export class Subscribe__Params {
  _event: Subscribe;

  constructor(event: Subscribe) {
    this._event = event;
  }

  get user(): Address {
    return this._event.parameters[0].value.toAddress();
  }

  get epoch(): BigInt {
    return this._event.parameters[1].value.toBigInt();
  }

  get start(): BigInt {
    return this._event.parameters[2].value.toBigInt();
  }

  get end(): BigInt {
    return this._event.parameters[3].value.toBigInt();
  }

  get rate(): BigInt {
    return this._event.parameters[4].value.toBigInt();
  }
}

export class TokensCollected extends ethereum.Event {
  get params(): TokensCollected__Params {
    return new TokensCollected__Params(this);
  }
}

export class TokensCollected__Params {
  _event: TokensCollected;

  constructor(event: TokensCollected) {
    this._event = event;
  }

  get owner(): Address {
    return this._event.parameters[0].value.toAddress();
  }

  get amount(): BigInt {
    return this._event.parameters[1].value.toBigInt();
  }

  get startEpoch(): BigInt {
    return this._event.parameters[2].value.toBigInt();
  }

  get endEpoch(): BigInt {
    return this._event.parameters[3].value.toBigInt();
  }
}

export class Unsubscribe extends ethereum.Event {
  get params(): Unsubscribe__Params {
    return new Unsubscribe__Params(this);
  }
}

export class Unsubscribe__Params {
  _event: Unsubscribe;

  constructor(event: Unsubscribe) {
    this._event = event;
  }

  get user(): Address {
    return this._event.parameters[0].value.toAddress();
  }

  get epoch(): BigInt {
    return this._event.parameters[1].value.toBigInt();
  }
}

export class Subscriptions__epochsResult {
  value0: BigInt;
  value1: BigInt;

  constructor(value0: BigInt, value1: BigInt) {
    this.value0 = value0;
    this.value1 = value1;
  }

  toMap(): TypedMap<string, ethereum.Value> {
    let map = new TypedMap<string, ethereum.Value>();
    map.set("value0", ethereum.Value.fromSignedBigInt(this.value0));
    map.set("value1", ethereum.Value.fromSignedBigInt(this.value1));
    return map;
  }

  getDelta(): BigInt {
    return this.value0;
  }

  getExtra(): BigInt {
    return this.value1;
  }
}

export class Subscriptions__pendingSubscriptionsResult {
  value0: BigInt;
  value1: BigInt;
  value2: BigInt;

  constructor(value0: BigInt, value1: BigInt, value2: BigInt) {
    this.value0 = value0;
    this.value1 = value1;
    this.value2 = value2;
  }

  toMap(): TypedMap<string, ethereum.Value> {
    let map = new TypedMap<string, ethereum.Value>();
    map.set("value0", ethereum.Value.fromUnsignedBigInt(this.value0));
    map.set("value1", ethereum.Value.fromUnsignedBigInt(this.value1));
    map.set("value2", ethereum.Value.fromUnsignedBigInt(this.value2));
    return map;
  }

  getStart(): BigInt {
    return this.value0;
  }

  getEnd(): BigInt {
    return this.value1;
  }

  getRate(): BigInt {
    return this.value2;
  }
}

export class Subscriptions__subscriptionsResult {
  value0: BigInt;
  value1: BigInt;
  value2: BigInt;

  constructor(value0: BigInt, value1: BigInt, value2: BigInt) {
    this.value0 = value0;
    this.value1 = value1;
    this.value2 = value2;
  }

  toMap(): TypedMap<string, ethereum.Value> {
    let map = new TypedMap<string, ethereum.Value>();
    map.set("value0", ethereum.Value.fromUnsignedBigInt(this.value0));
    map.set("value1", ethereum.Value.fromUnsignedBigInt(this.value1));
    map.set("value2", ethereum.Value.fromUnsignedBigInt(this.value2));
    return map;
  }

  getStart(): BigInt {
    return this.value0;
  }

  getEnd(): BigInt {
    return this.value1;
  }

  getRate(): BigInt {
    return this.value2;
  }
}

export class Subscriptions extends ethereum.SmartContract {
  static bind(address: Address): Subscriptions {
    return new Subscriptions("Subscriptions", address);
  }

  authorizedSigners(param0: Address, param1: Address): boolean {
    let result = super.call(
      "authorizedSigners",
      "authorizedSigners(address,address):(bool)",
      [ethereum.Value.fromAddress(param0), ethereum.Value.fromAddress(param1)]
    );

    return result[0].toBoolean();
  }

  try_authorizedSigners(
    param0: Address,
    param1: Address
  ): ethereum.CallResult<boolean> {
    let result = super.tryCall(
      "authorizedSigners",
      "authorizedSigners(address,address):(bool)",
      [ethereum.Value.fromAddress(param0), ethereum.Value.fromAddress(param1)]
    );
    if (result.reverted) {
      return new ethereum.CallResult();
    }
    let value = result.value;
    return ethereum.CallResult.fromValue(value[0].toBoolean());
  }

  checkAuthorizedSigner(_user: Address, _signer: Address): boolean {
    let result = super.call(
      "checkAuthorizedSigner",
      "checkAuthorizedSigner(address,address):(bool)",
      [ethereum.Value.fromAddress(_user), ethereum.Value.fromAddress(_signer)]
    );

    return result[0].toBoolean();
  }

  try_checkAuthorizedSigner(
    _user: Address,
    _signer: Address
  ): ethereum.CallResult<boolean> {
    let result = super.tryCall(
      "checkAuthorizedSigner",
      "checkAuthorizedSigner(address,address):(bool)",
      [ethereum.Value.fromAddress(_user), ethereum.Value.fromAddress(_signer)]
    );
    if (result.reverted) {
      return new ethereum.CallResult();
    }
    let value = result.value;
    return ethereum.CallResult.fromValue(value[0].toBoolean());
  }

  collectPerEpoch(): BigInt {
    let result = super.call(
      "collectPerEpoch",
      "collectPerEpoch():(int128)",
      []
    );

    return result[0].toBigInt();
  }

  try_collectPerEpoch(): ethereum.CallResult<BigInt> {
    let result = super.tryCall(
      "collectPerEpoch",
      "collectPerEpoch():(int128)",
      []
    );
    if (result.reverted) {
      return new ethereum.CallResult();
    }
    let value = result.value;
    return ethereum.CallResult.fromValue(value[0].toBigInt());
  }

  currentEpoch(): BigInt {
    let result = super.call("currentEpoch", "currentEpoch():(uint256)", []);

    return result[0].toBigInt();
  }

  try_currentEpoch(): ethereum.CallResult<BigInt> {
    let result = super.tryCall("currentEpoch", "currentEpoch():(uint256)", []);
    if (result.reverted) {
      return new ethereum.CallResult();
    }
    let value = result.value;
    return ethereum.CallResult.fromValue(value[0].toBigInt());
  }

  epochSeconds(): BigInt {
    let result = super.call("epochSeconds", "epochSeconds():(uint64)", []);

    return result[0].toBigInt();
  }

  try_epochSeconds(): ethereum.CallResult<BigInt> {
    let result = super.tryCall("epochSeconds", "epochSeconds():(uint64)", []);
    if (result.reverted) {
      return new ethereum.CallResult();
    }
    let value = result.value;
    return ethereum.CallResult.fromValue(value[0].toBigInt());
  }

  epochs(param0: BigInt): Subscriptions__epochsResult {
    let result = super.call("epochs", "epochs(uint256):(int128,int128)", [
      ethereum.Value.fromUnsignedBigInt(param0)
    ]);

    return new Subscriptions__epochsResult(
      result[0].toBigInt(),
      result[1].toBigInt()
    );
  }

  try_epochs(param0: BigInt): ethereum.CallResult<Subscriptions__epochsResult> {
    let result = super.tryCall("epochs", "epochs(uint256):(int128,int128)", [
      ethereum.Value.fromUnsignedBigInt(param0)
    ]);
    if (result.reverted) {
      return new ethereum.CallResult();
    }
    let value = result.value;
    return ethereum.CallResult.fromValue(
      new Subscriptions__epochsResult(value[0].toBigInt(), value[1].toBigInt())
    );
  }

  locked(_subStart: BigInt, _subEnd: BigInt, _subRate: BigInt): BigInt {
    let result = super.call(
      "locked",
      "locked(uint64,uint64,uint128):(uint128)",
      [
        ethereum.Value.fromUnsignedBigInt(_subStart),
        ethereum.Value.fromUnsignedBigInt(_subEnd),
        ethereum.Value.fromUnsignedBigInt(_subRate)
      ]
    );

    return result[0].toBigInt();
  }

  try_locked(
    _subStart: BigInt,
    _subEnd: BigInt,
    _subRate: BigInt
  ): ethereum.CallResult<BigInt> {
    let result = super.tryCall(
      "locked",
      "locked(uint64,uint64,uint128):(uint128)",
      [
        ethereum.Value.fromUnsignedBigInt(_subStart),
        ethereum.Value.fromUnsignedBigInt(_subEnd),
        ethereum.Value.fromUnsignedBigInt(_subRate)
      ]
    );
    if (result.reverted) {
      return new ethereum.CallResult();
    }
    let value = result.value;
    return ethereum.CallResult.fromValue(value[0].toBigInt());
  }

  locked1(_user: Address): BigInt {
    let result = super.call("locked", "locked(address):(uint128)", [
      ethereum.Value.fromAddress(_user)
    ]);

    return result[0].toBigInt();
  }

  try_locked1(_user: Address): ethereum.CallResult<BigInt> {
    let result = super.tryCall("locked", "locked(address):(uint128)", [
      ethereum.Value.fromAddress(_user)
    ]);
    if (result.reverted) {
      return new ethereum.CallResult();
    }
    let value = result.value;
    return ethereum.CallResult.fromValue(value[0].toBigInt());
  }

  owner(): Address {
    let result = super.call("owner", "owner():(address)", []);

    return result[0].toAddress();
  }

  try_owner(): ethereum.CallResult<Address> {
    let result = super.tryCall("owner", "owner():(address)", []);
    if (result.reverted) {
      return new ethereum.CallResult();
    }
    let value = result.value;
    return ethereum.CallResult.fromValue(value[0].toAddress());
  }

  pendingSubscriptions(
    param0: Address
  ): Subscriptions__pendingSubscriptionsResult {
    let result = super.call(
      "pendingSubscriptions",
      "pendingSubscriptions(address):(uint64,uint64,uint128)",
      [ethereum.Value.fromAddress(param0)]
    );

    return new Subscriptions__pendingSubscriptionsResult(
      result[0].toBigInt(),
      result[1].toBigInt(),
      result[2].toBigInt()
    );
  }

  try_pendingSubscriptions(
    param0: Address
  ): ethereum.CallResult<Subscriptions__pendingSubscriptionsResult> {
    let result = super.tryCall(
      "pendingSubscriptions",
      "pendingSubscriptions(address):(uint64,uint64,uint128)",
      [ethereum.Value.fromAddress(param0)]
    );
    if (result.reverted) {
      return new ethereum.CallResult();
    }
    let value = result.value;
    return ethereum.CallResult.fromValue(
      new Subscriptions__pendingSubscriptionsResult(
        value[0].toBigInt(),
        value[1].toBigInt(),
        value[2].toBigInt()
      )
    );
  }

  subscriptions(param0: Address): Subscriptions__subscriptionsResult {
    let result = super.call(
      "subscriptions",
      "subscriptions(address):(uint64,uint64,uint128)",
      [ethereum.Value.fromAddress(param0)]
    );

    return new Subscriptions__subscriptionsResult(
      result[0].toBigInt(),
      result[1].toBigInt(),
      result[2].toBigInt()
    );
  }

  try_subscriptions(
    param0: Address
  ): ethereum.CallResult<Subscriptions__subscriptionsResult> {
    let result = super.tryCall(
      "subscriptions",
      "subscriptions(address):(uint64,uint64,uint128)",
      [ethereum.Value.fromAddress(param0)]
    );
    if (result.reverted) {
      return new ethereum.CallResult();
    }
    let value = result.value;
    return ethereum.CallResult.fromValue(
      new Subscriptions__subscriptionsResult(
        value[0].toBigInt(),
        value[1].toBigInt(),
        value[2].toBigInt()
      )
    );
  }

  timestampToEpoch(_timestamp: BigInt): BigInt {
    let result = super.call(
      "timestampToEpoch",
      "timestampToEpoch(uint256):(uint256)",
      [ethereum.Value.fromUnsignedBigInt(_timestamp)]
    );

    return result[0].toBigInt();
  }

  try_timestampToEpoch(_timestamp: BigInt): ethereum.CallResult<BigInt> {
    let result = super.tryCall(
      "timestampToEpoch",
      "timestampToEpoch(uint256):(uint256)",
      [ethereum.Value.fromUnsignedBigInt(_timestamp)]
    );
    if (result.reverted) {
      return new ethereum.CallResult();
    }
    let value = result.value;
    return ethereum.CallResult.fromValue(value[0].toBigInt());
  }

  token(): Address {
    let result = super.call("token", "token():(address)", []);

    return result[0].toAddress();
  }

  try_token(): ethereum.CallResult<Address> {
    let result = super.tryCall("token", "token():(address)", []);
    if (result.reverted) {
      return new ethereum.CallResult();
    }
    let value = result.value;
    return ethereum.CallResult.fromValue(value[0].toAddress());
  }

  uncollectedEpoch(): BigInt {
    let result = super.call(
      "uncollectedEpoch",
      "uncollectedEpoch():(uint256)",
      []
    );

    return result[0].toBigInt();
  }

  try_uncollectedEpoch(): ethereum.CallResult<BigInt> {
    let result = super.tryCall(
      "uncollectedEpoch",
      "uncollectedEpoch():(uint256)",
      []
    );
    if (result.reverted) {
      return new ethereum.CallResult();
    }
    let value = result.value;
    return ethereum.CallResult.fromValue(value[0].toBigInt());
  }

  unlocked(_subStart: BigInt, _subEnd: BigInt, _subRate: BigInt): BigInt {
    let result = super.call(
      "unlocked",
      "unlocked(uint64,uint64,uint128):(uint128)",
      [
        ethereum.Value.fromUnsignedBigInt(_subStart),
        ethereum.Value.fromUnsignedBigInt(_subEnd),
        ethereum.Value.fromUnsignedBigInt(_subRate)
      ]
    );

    return result[0].toBigInt();
  }

  try_unlocked(
    _subStart: BigInt,
    _subEnd: BigInt,
    _subRate: BigInt
  ): ethereum.CallResult<BigInt> {
    let result = super.tryCall(
      "unlocked",
      "unlocked(uint64,uint64,uint128):(uint128)",
      [
        ethereum.Value.fromUnsignedBigInt(_subStart),
        ethereum.Value.fromUnsignedBigInt(_subEnd),
        ethereum.Value.fromUnsignedBigInt(_subRate)
      ]
    );
    if (result.reverted) {
      return new ethereum.CallResult();
    }
    let value = result.value;
    return ethereum.CallResult.fromValue(value[0].toBigInt());
  }

  unlocked1(_user: Address): BigInt {
    let result = super.call("unlocked", "unlocked(address):(uint128)", [
      ethereum.Value.fromAddress(_user)
    ]);

    return result[0].toBigInt();
  }

  try_unlocked1(_user: Address): ethereum.CallResult<BigInt> {
    let result = super.tryCall("unlocked", "unlocked(address):(uint128)", [
      ethereum.Value.fromAddress(_user)
    ]);
    if (result.reverted) {
      return new ethereum.CallResult();
    }
    let value = result.value;
    return ethereum.CallResult.fromValue(value[0].toBigInt());
  }
}

export class ConstructorCall extends ethereum.Call {
  get inputs(): ConstructorCall__Inputs {
    return new ConstructorCall__Inputs(this);
  }

  get outputs(): ConstructorCall__Outputs {
    return new ConstructorCall__Outputs(this);
  }
}

export class ConstructorCall__Inputs {
  _call: ConstructorCall;

  constructor(call: ConstructorCall) {
    this._call = call;
  }

  get _token(): Address {
    return this._call.inputValues[0].value.toAddress();
  }

  get _epochSeconds(): BigInt {
    return this._call.inputValues[1].value.toBigInt();
  }
}

export class ConstructorCall__Outputs {
  _call: ConstructorCall;

  constructor(call: ConstructorCall) {
    this._call = call;
  }
}

export class AddAuthorizedSignerCall extends ethereum.Call {
  get inputs(): AddAuthorizedSignerCall__Inputs {
    return new AddAuthorizedSignerCall__Inputs(this);
  }

  get outputs(): AddAuthorizedSignerCall__Outputs {
    return new AddAuthorizedSignerCall__Outputs(this);
  }
}

export class AddAuthorizedSignerCall__Inputs {
  _call: AddAuthorizedSignerCall;

  constructor(call: AddAuthorizedSignerCall) {
    this._call = call;
  }

  get _user(): Address {
    return this._call.inputValues[0].value.toAddress();
  }

  get _signer(): Address {
    return this._call.inputValues[1].value.toAddress();
  }
}

export class AddAuthorizedSignerCall__Outputs {
  _call: AddAuthorizedSignerCall;

  constructor(call: AddAuthorizedSignerCall) {
    this._call = call;
  }
}

export class CollectCall extends ethereum.Call {
  get inputs(): CollectCall__Inputs {
    return new CollectCall__Inputs(this);
  }

  get outputs(): CollectCall__Outputs {
    return new CollectCall__Outputs(this);
  }
}

export class CollectCall__Inputs {
  _call: CollectCall;

  constructor(call: CollectCall) {
    this._call = call;
  }

  get _offset(): BigInt {
    return this._call.inputValues[0].value.toBigInt();
  }
}

export class CollectCall__Outputs {
  _call: CollectCall;

  constructor(call: CollectCall) {
    this._call = call;
  }
}

export class Collect1Call extends ethereum.Call {
  get inputs(): Collect1Call__Inputs {
    return new Collect1Call__Inputs(this);
  }

  get outputs(): Collect1Call__Outputs {
    return new Collect1Call__Outputs(this);
  }
}

export class Collect1Call__Inputs {
  _call: Collect1Call;

  constructor(call: Collect1Call) {
    this._call = call;
  }
}

export class Collect1Call__Outputs {
  _call: Collect1Call;

  constructor(call: Collect1Call) {
    this._call = call;
  }
}

export class ExtendSubscriptionCall extends ethereum.Call {
  get inputs(): ExtendSubscriptionCall__Inputs {
    return new ExtendSubscriptionCall__Inputs(this);
  }

  get outputs(): ExtendSubscriptionCall__Outputs {
    return new ExtendSubscriptionCall__Outputs(this);
  }
}

export class ExtendSubscriptionCall__Inputs {
  _call: ExtendSubscriptionCall;

  constructor(call: ExtendSubscriptionCall) {
    this._call = call;
  }

  get user(): Address {
    return this._call.inputValues[0].value.toAddress();
  }

  get end(): BigInt {
    return this._call.inputValues[1].value.toBigInt();
  }
}

export class ExtendSubscriptionCall__Outputs {
  _call: ExtendSubscriptionCall;

  constructor(call: ExtendSubscriptionCall) {
    this._call = call;
  }
}

export class FulfilCall extends ethereum.Call {
  get inputs(): FulfilCall__Inputs {
    return new FulfilCall__Inputs(this);
  }

  get outputs(): FulfilCall__Outputs {
    return new FulfilCall__Outputs(this);
  }
}

export class FulfilCall__Inputs {
  _call: FulfilCall;

  constructor(call: FulfilCall) {
    this._call = call;
  }

  get _to(): Address {
    return this._call.inputValues[0].value.toAddress();
  }

  get _amount(): BigInt {
    return this._call.inputValues[1].value.toBigInt();
  }
}

export class FulfilCall__Outputs {
  _call: FulfilCall;

  constructor(call: FulfilCall) {
    this._call = call;
  }
}

export class RemoveAuthorizedSignerCall extends ethereum.Call {
  get inputs(): RemoveAuthorizedSignerCall__Inputs {
    return new RemoveAuthorizedSignerCall__Inputs(this);
  }

  get outputs(): RemoveAuthorizedSignerCall__Outputs {
    return new RemoveAuthorizedSignerCall__Outputs(this);
  }
}

export class RemoveAuthorizedSignerCall__Inputs {
  _call: RemoveAuthorizedSignerCall;

  constructor(call: RemoveAuthorizedSignerCall) {
    this._call = call;
  }

  get _user(): Address {
    return this._call.inputValues[0].value.toAddress();
  }

  get _signer(): Address {
    return this._call.inputValues[1].value.toAddress();
  }
}

export class RemoveAuthorizedSignerCall__Outputs {
  _call: RemoveAuthorizedSignerCall;

  constructor(call: RemoveAuthorizedSignerCall) {
    this._call = call;
  }
}

export class RenounceOwnershipCall extends ethereum.Call {
  get inputs(): RenounceOwnershipCall__Inputs {
    return new RenounceOwnershipCall__Inputs(this);
  }

  get outputs(): RenounceOwnershipCall__Outputs {
    return new RenounceOwnershipCall__Outputs(this);
  }
}

export class RenounceOwnershipCall__Inputs {
  _call: RenounceOwnershipCall;

  constructor(call: RenounceOwnershipCall) {
    this._call = call;
  }
}

export class RenounceOwnershipCall__Outputs {
  _call: RenounceOwnershipCall;

  constructor(call: RenounceOwnershipCall) {
    this._call = call;
  }
}

export class SetPendingSubscriptionCall extends ethereum.Call {
  get inputs(): SetPendingSubscriptionCall__Inputs {
    return new SetPendingSubscriptionCall__Inputs(this);
  }

  get outputs(): SetPendingSubscriptionCall__Outputs {
    return new SetPendingSubscriptionCall__Outputs(this);
  }
}

export class SetPendingSubscriptionCall__Inputs {
  _call: SetPendingSubscriptionCall;

  constructor(call: SetPendingSubscriptionCall) {
    this._call = call;
  }

  get user(): Address {
    return this._call.inputValues[0].value.toAddress();
  }

  get start(): BigInt {
    return this._call.inputValues[1].value.toBigInt();
  }

  get end(): BigInt {
    return this._call.inputValues[2].value.toBigInt();
  }

  get rate(): BigInt {
    return this._call.inputValues[3].value.toBigInt();
  }
}

export class SetPendingSubscriptionCall__Outputs {
  _call: SetPendingSubscriptionCall;

  constructor(call: SetPendingSubscriptionCall) {
    this._call = call;
  }
}

export class SubscribeCall extends ethereum.Call {
  get inputs(): SubscribeCall__Inputs {
    return new SubscribeCall__Inputs(this);
  }

  get outputs(): SubscribeCall__Outputs {
    return new SubscribeCall__Outputs(this);
  }
}

export class SubscribeCall__Inputs {
  _call: SubscribeCall;

  constructor(call: SubscribeCall) {
    this._call = call;
  }

  get user(): Address {
    return this._call.inputValues[0].value.toAddress();
  }

  get start(): BigInt {
    return this._call.inputValues[1].value.toBigInt();
  }

  get end(): BigInt {
    return this._call.inputValues[2].value.toBigInt();
  }

  get rate(): BigInt {
    return this._call.inputValues[3].value.toBigInt();
  }
}

export class SubscribeCall__Outputs {
  _call: SubscribeCall;

  constructor(call: SubscribeCall) {
    this._call = call;
  }
}

export class TransferOwnershipCall extends ethereum.Call {
  get inputs(): TransferOwnershipCall__Inputs {
    return new TransferOwnershipCall__Inputs(this);
  }

  get outputs(): TransferOwnershipCall__Outputs {
    return new TransferOwnershipCall__Outputs(this);
  }
}

export class TransferOwnershipCall__Inputs {
  _call: TransferOwnershipCall;

  constructor(call: TransferOwnershipCall) {
    this._call = call;
  }

  get newOwner(): Address {
    return this._call.inputValues[0].value.toAddress();
  }
}

export class TransferOwnershipCall__Outputs {
  _call: TransferOwnershipCall;

  constructor(call: TransferOwnershipCall) {
    this._call = call;
  }
}

export class UnsubscribeCall extends ethereum.Call {
  get inputs(): UnsubscribeCall__Inputs {
    return new UnsubscribeCall__Inputs(this);
  }

  get outputs(): UnsubscribeCall__Outputs {
    return new UnsubscribeCall__Outputs(this);
  }
}

export class UnsubscribeCall__Inputs {
  _call: UnsubscribeCall;

  constructor(call: UnsubscribeCall) {
    this._call = call;
  }
}

export class UnsubscribeCall__Outputs {
  _call: UnsubscribeCall;

  constructor(call: UnsubscribeCall) {
    this._call = call;
  }
}
