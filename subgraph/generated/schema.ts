// THIS IS AN AUTOGENERATED FILE. DO NOT EDIT THIS FILE DIRECTLY.

import {
  TypedMap,
  Entity,
  Value,
  ValueKind,
  store,
  Bytes,
  BigInt,
  BigDecimal
} from "@graphprotocol/graph-ts";

export class Init extends Entity {
  constructor(id: Bytes) {
    super();
    this.set("id", Value.fromBytes(id));
  }

  save(): void {
    let id = this.get("id");
    assert(id != null, "Cannot save Init entity without an ID");
    if (id) {
      assert(
        id.kind == ValueKind.BYTES,
        `Entities of type Init must have an ID of type Bytes but the id '${id.displayData()}' is of type ${id.displayKind()}`
      );
      store.set("Init", id.toBytes().toHexString(), this);
    }
  }

  static load(id: Bytes): Init | null {
    return changetype<Init | null>(store.get("Init", id.toHexString()));
  }

  get id(): Bytes {
    let value = this.get("id");
    return value!.toBytes();
  }

  set id(value: Bytes) {
    this.set("id", Value.fromBytes(value));
  }

  get blockNumber(): BigInt {
    let value = this.get("blockNumber");
    return value!.toBigInt();
  }

  set blockNumber(value: BigInt) {
    this.set("blockNumber", Value.fromBigInt(value));
  }

  get blockTimestamp(): BigInt {
    let value = this.get("blockTimestamp");
    return value!.toBigInt();
  }

  set blockTimestamp(value: BigInt) {
    this.set("blockTimestamp", Value.fromBigInt(value));
  }

  get transactionHash(): Bytes {
    let value = this.get("transactionHash");
    return value!.toBytes();
  }

  set transactionHash(value: Bytes) {
    this.set("transactionHash", Value.fromBytes(value));
  }

  get token(): Bytes {
    let value = this.get("token");
    return value!.toBytes();
  }

  set token(value: Bytes) {
    this.set("token", Value.fromBytes(value));
  }
}

export class User extends Entity {
  constructor(id: Bytes) {
    super();
    this.set("id", Value.fromBytes(id));
  }

  save(): void {
    let id = this.get("id");
    assert(id != null, "Cannot save User entity without an ID");
    if (id) {
      assert(
        id.kind == ValueKind.BYTES,
        `Entities of type User must have an ID of type Bytes but the id '${id.displayData()}' is of type ${id.displayKind()}`
      );
      store.set("User", id.toBytes().toHexString(), this);
    }
  }

  static load(id: Bytes): User | null {
    return changetype<User | null>(store.get("User", id.toHexString()));
  }

  get id(): Bytes {
    let value = this.get("id");
    return value!.toBytes();
  }

  set id(value: Bytes) {
    this.set("id", Value.fromBytes(value));
  }

  get subscribeEvents(): Array<Bytes> {
    let value = this.get("subscribeEvents");
    return value!.toBytesArray();
  }

  set subscribeEvents(value: Array<Bytes>) {
    this.set("subscribeEvents", Value.fromBytesArray(value));
  }

  get unsubscribeEvents(): Array<Bytes> | null {
    let value = this.get("unsubscribeEvents");
    if (!value || value.kind == ValueKind.NULL) {
      return null;
    } else {
      return value.toBytesArray();
    }
  }

  set unsubscribeEvents(value: Array<Bytes> | null) {
    if (!value) {
      this.unset("unsubscribeEvents");
    } else {
      this.set("unsubscribeEvents", Value.fromBytesArray(<Array<Bytes>>value));
    }
  }

  get extendEvents(): Array<Bytes> | null {
    let value = this.get("extendEvents");
    if (!value || value.kind == ValueKind.NULL) {
      return null;
    } else {
      return value.toBytesArray();
    }
  }

  set extendEvents(value: Array<Bytes> | null) {
    if (!value) {
      this.unset("extendEvents");
    } else {
      this.set("extendEvents", Value.fromBytesArray(<Array<Bytes>>value));
    }
  }

  get subscriptions(): Array<Bytes> | null {
    let value = this.get("subscriptions");
    if (!value || value.kind == ValueKind.NULL) {
      return null;
    } else {
      return value.toBytesArray();
    }
  }

  set subscriptions(value: Array<Bytes> | null) {
    if (!value) {
      this.unset("subscriptions");
    } else {
      this.set(
        "subscriptions",
        Value.fromBytesArray(<Array<Bytes>>value)
      );
    }
  }

  get authorizedSigners(): Array<Bytes> | null {
    let value = this.get("authorizedSigners");
    if (!value || value.kind == ValueKind.NULL) {
      return null;
    } else {
      return value.toBytesArray();
    }
  }

  set authorizedSigners(value: Array<Bytes> | null) {
    if (!value) {
      this.unset("authorizedSigners");
    } else {
      this.set("authorizedSigners", Value.fromBytesArray(<Array<Bytes>>value));
    }
  }

  get eventCount(): i32 {
    let value = this.get("eventCount");
    return value!.toI32();
  }

  set eventCount(value: i32) {
    this.set("eventCount", Value.fromI32(value));
  }

  get events(): Array<Bytes> | null {
    let value = this.get("events");
    if (!value || value.kind == ValueKind.NULL) {
      return null;
    } else {
      return value.toBytesArray();
    }
  }

  set events(value: Array<Bytes> | null) {
    if (!value) {
      this.unset("events");
    } else {
      this.set("events", Value.fromBytesArray(<Array<Bytes>>value));
    }
  }
}

export class Subscribe extends Entity {
  constructor(id: Bytes) {
    super();
    this.set("id", Value.fromBytes(id));
  }

  save(): void {
    let id = this.get("id");
    assert(id != null, "Cannot save Subscribe entity without an ID");
    if (id) {
      assert(
        id.kind == ValueKind.BYTES,
        `Entities of type Subscribe must have an ID of type Bytes but the id '${id.displayData()}' is of type ${id.displayKind()}`
      );
      store.set("Subscribe", id.toBytes().toHexString(), this);
    }
  }

  static load(id: Bytes): Subscribe | null {
    return changetype<Subscribe | null>(
      store.get("Subscribe", id.toHexString())
    );
  }

  get id(): Bytes {
    let value = this.get("id");
    return value!.toBytes();
  }

  set id(value: Bytes) {
    this.set("id", Value.fromBytes(value));
  }

  get blockNumber(): BigInt {
    let value = this.get("blockNumber");
    return value!.toBigInt();
  }

  set blockNumber(value: BigInt) {
    this.set("blockNumber", Value.fromBigInt(value));
  }

  get blockTimestamp(): BigInt {
    let value = this.get("blockTimestamp");
    return value!.toBigInt();
  }

  set blockTimestamp(value: BigInt) {
    this.set("blockTimestamp", Value.fromBigInt(value));
  }

  get transactionHash(): Bytes {
    let value = this.get("transactionHash");
    return value!.toBytes();
  }

  set transactionHash(value: Bytes) {
    this.set("transactionHash", Value.fromBytes(value));
  }

  get user(): Bytes {
    let value = this.get("user");
    return value!.toBytes();
  }

  set user(value: Bytes) {
    this.set("user", Value.fromBytes(value));
  }

  get start(): BigInt {
    let value = this.get("start");
    return value!.toBigInt();
  }

  set start(value: BigInt) {
    this.set("start", Value.fromBigInt(value));
  }

  get end(): BigInt {
    let value = this.get("end");
    return value!.toBigInt();
  }

  set end(value: BigInt) {
    this.set("end", Value.fromBigInt(value));
  }

  get rate(): BigInt {
    let value = this.get("rate");
    return value!.toBigInt();
  }

  set rate(value: BigInt) {
    this.set("rate", Value.fromBigInt(value));
  }
}

export class Unsubscribe extends Entity {
  constructor(id: Bytes) {
    super();
    this.set("id", Value.fromBytes(id));
  }

  save(): void {
    let id = this.get("id");
    assert(id != null, "Cannot save Unsubscribe entity without an ID");
    if (id) {
      assert(
        id.kind == ValueKind.BYTES,
        `Entities of type Unsubscribe must have an ID of type Bytes but the id '${id.displayData()}' is of type ${id.displayKind()}`
      );
      store.set("Unsubscribe", id.toBytes().toHexString(), this);
    }
  }

  static load(id: Bytes): Unsubscribe | null {
    return changetype<Unsubscribe | null>(
      store.get("Unsubscribe", id.toHexString())
    );
  }

  get id(): Bytes {
    let value = this.get("id");
    return value!.toBytes();
  }

  set id(value: Bytes) {
    this.set("id", Value.fromBytes(value));
  }

  get blockNumber(): BigInt {
    let value = this.get("blockNumber");
    return value!.toBigInt();
  }

  set blockNumber(value: BigInt) {
    this.set("blockNumber", Value.fromBigInt(value));
  }

  get blockTimestamp(): BigInt {
    let value = this.get("blockTimestamp");
    return value!.toBigInt();
  }

  set blockTimestamp(value: BigInt) {
    this.set("blockTimestamp", Value.fromBigInt(value));
  }

  get transactionHash(): Bytes {
    let value = this.get("transactionHash");
    return value!.toBytes();
  }

  set transactionHash(value: Bytes) {
    this.set("transactionHash", Value.fromBytes(value));
  }

  get user(): Bytes {
    let value = this.get("user");
    return value!.toBytes();
  }

  set user(value: Bytes) {
    this.set("user", Value.fromBytes(value));
  }
}

export class Extend extends Entity {
  constructor(id: Bytes) {
    super();
    this.set("id", Value.fromBytes(id));
  }

  save(): void {
    let id = this.get("id");
    assert(id != null, "Cannot save Extend entity without an ID");
    if (id) {
      assert(
        id.kind == ValueKind.BYTES,
        `Entities of type Extend must have an ID of type Bytes but the id '${id.displayData()}' is of type ${id.displayKind()}`
      );
      store.set("Extend", id.toBytes().toHexString(), this);
    }
  }

  static load(id: Bytes): Extend | null {
    return changetype<Extend | null>(store.get("Extend", id.toHexString()));
  }

  get id(): Bytes {
    let value = this.get("id");
    return value!.toBytes();
  }

  set id(value: Bytes) {
    this.set("id", Value.fromBytes(value));
  }

  get blockNumber(): BigInt {
    let value = this.get("blockNumber");
    return value!.toBigInt();
  }

  set blockNumber(value: BigInt) {
    this.set("blockNumber", Value.fromBigInt(value));
  }

  get blockTimestamp(): BigInt {
    let value = this.get("blockTimestamp");
    return value!.toBigInt();
  }

  set blockTimestamp(value: BigInt) {
    this.set("blockTimestamp", Value.fromBigInt(value));
  }

  get transactionHash(): Bytes {
    let value = this.get("transactionHash");
    return value!.toBytes();
  }

  set transactionHash(value: Bytes) {
    this.set("transactionHash", Value.fromBytes(value));
  }

  get user(): Bytes {
    let value = this.get("user");
    return value!.toBytes();
  }

  set user(value: Bytes) {
    this.set("user", Value.fromBytes(value));
  }

  get end(): BigInt {
    let value = this.get("end");
    return value!.toBigInt();
  }

  set end(value: BigInt) {
    this.set("end", Value.fromBigInt(value));
  }
}

export class Subscription extends Entity {
  constructor(id: Bytes) {
    super();
    this.set("id", Value.fromBytes(id));
  }

  save(): void {
    let id = this.get("id");
    assert(id != null, "Cannot save Subscription entity without an ID");
    if (id) {
      assert(
        id.kind == ValueKind.BYTES,
        `Entities of type Subscription must have an ID of type Bytes but the id '${id.displayData()}' is of type ${id.displayKind()}`
      );
      store.set("Subscription", id.toBytes().toHexString(), this);
    }
  }

  static load(id: Bytes): Subscription | null {
    return changetype<Subscription | null>(
      store.get("Subscription", id.toHexString())
    );
  }

  get id(): Bytes {
    let value = this.get("id");
    return value!.toBytes();
  }

  set id(value: Bytes) {
    this.set("id", Value.fromBytes(value));
  }

  get user(): Bytes {
    let value = this.get("user");
    return value!.toBytes();
  }

  set user(value: Bytes) {
    this.set("user", Value.fromBytes(value));
  }

  get start(): BigInt {
    let value = this.get("start");
    return value!.toBigInt();
  }

  set start(value: BigInt) {
    this.set("start", Value.fromBigInt(value));
  }

  get end(): BigInt {
    let value = this.get("end");
    return value!.toBigInt();
  }

  set end(value: BigInt) {
    this.set("end", Value.fromBigInt(value));
  }

  get rate(): BigInt {
    let value = this.get("rate");
    return value!.toBigInt();
  }

  set rate(value: BigInt) {
    this.set("rate", Value.fromBigInt(value));
  }
}

export class AuthorizedSigner extends Entity {
  constructor(id: Bytes) {
    super();
    this.set("id", Value.fromBytes(id));
  }

  save(): void {
    let id = this.get("id");
    assert(id != null, "Cannot save AuthorizedSigner entity without an ID");
    if (id) {
      assert(
        id.kind == ValueKind.BYTES,
        `Entities of type AuthorizedSigner must have an ID of type Bytes but the id '${id.displayData()}' is of type ${id.displayKind()}`
      );
      store.set("AuthorizedSigner", id.toBytes().toHexString(), this);
    }
  }

  static load(id: Bytes): AuthorizedSigner | null {
    return changetype<AuthorizedSigner | null>(
      store.get("AuthorizedSigner", id.toHexString())
    );
  }

  get id(): Bytes {
    let value = this.get("id");
    return value!.toBytes();
  }

  set id(value: Bytes) {
    this.set("id", Value.fromBytes(value));
  }

  get user(): Bytes {
    let value = this.get("user");
    return value!.toBytes();
  }

  set user(value: Bytes) {
    this.set("user", Value.fromBytes(value));
  }

  get signer(): Bytes {
    let value = this.get("signer");
    return value!.toBytes();
  }

  set signer(value: Bytes) {
    this.set("signer", Value.fromBytes(value));
  }
}

export class UserSubscriptionCreatedEvent extends Entity {
  constructor(id: Bytes) {
    super();
    this.set("id", Value.fromBytes(id));
  }

  save(): void {
    let id = this.get("id");
    assert(
      id != null,
      "Cannot save UserSubscriptionCreatedEvent entity without an ID"
    );
    if (id) {
      assert(
        id.kind == ValueKind.BYTES,
        `Entities of type UserSubscriptionCreatedEvent must have an ID of type Bytes but the id '${id.displayData()}' is of type ${id.displayKind()}`
      );
      store.set(
        "UserSubscriptionCreatedEvent",
        id.toBytes().toHexString(),
        this
      );
    }
  }

  static load(id: Bytes): UserSubscriptionCreatedEvent | null {
    return changetype<UserSubscriptionCreatedEvent | null>(
      store.get("UserSubscriptionCreatedEvent", id.toHexString())
    );
  }

  get id(): Bytes {
    let value = this.get("id");
    return value!.toBytes();
  }

  set id(value: Bytes) {
    this.set("id", Value.fromBytes(value));
  }

  get user(): Bytes {
    let value = this.get("user");
    return value!.toBytes();
  }

  set user(value: Bytes) {
    this.set("user", Value.fromBytes(value));
  }

  get blockNumber(): BigInt {
    let value = this.get("blockNumber");
    return value!.toBigInt();
  }

  set blockNumber(value: BigInt) {
    this.set("blockNumber", Value.fromBigInt(value));
  }

  get blockTimestamp(): BigInt {
    let value = this.get("blockTimestamp");
    return value!.toBigInt();
  }

  set blockTimestamp(value: BigInt) {
    this.set("blockTimestamp", Value.fromBigInt(value));
  }

  get txHash(): Bytes {
    let value = this.get("txHash");
    return value!.toBytes();
  }

  set txHash(value: Bytes) {
    this.set("txHash", Value.fromBytes(value));
  }

  get eventType(): string {
    let value = this.get("eventType");
    return value!.toString();
  }

  set eventType(value: string) {
    this.set("eventType", Value.fromString(value));
  }

  get currentSubscriptionStart(): BigInt {
    let value = this.get("currentSubscriptionStart");
    return value!.toBigInt();
  }

  set currentSubscriptionStart(value: BigInt) {
    this.set("currentSubscriptionStart", Value.fromBigInt(value));
  }

  get currentSubscriptionEnd(): BigInt {
    let value = this.get("currentSubscriptionEnd");
    return value!.toBigInt();
  }

  set currentSubscriptionEnd(value: BigInt) {
    this.set("currentSubscriptionEnd", Value.fromBigInt(value));
  }

  get currentSubscriptionRate(): BigInt {
    let value = this.get("currentSubscriptionRate");
    return value!.toBigInt();
  }

  set currentSubscriptionRate(value: BigInt) {
    this.set("currentSubscriptionRate", Value.fromBigInt(value));
  }
}

export class UserSubscriptionCanceledEvent extends Entity {
  constructor(id: Bytes) {
    super();
    this.set("id", Value.fromBytes(id));
  }

  save(): void {
    let id = this.get("id");
    assert(
      id != null,
      "Cannot save UserSubscriptionCanceledEvent entity without an ID"
    );
    if (id) {
      assert(
        id.kind == ValueKind.BYTES,
        `Entities of type UserSubscriptionCanceledEvent must have an ID of type Bytes but the id '${id.displayData()}' is of type ${id.displayKind()}`
      );
      store.set(
        "UserSubscriptionCanceledEvent",
        id.toBytes().toHexString(),
        this
      );
    }
  }

  static load(id: Bytes): UserSubscriptionCanceledEvent | null {
    return changetype<UserSubscriptionCanceledEvent | null>(
      store.get("UserSubscriptionCanceledEvent", id.toHexString())
    );
  }

  get id(): Bytes {
    let value = this.get("id");
    return value!.toBytes();
  }

  set id(value: Bytes) {
    this.set("id", Value.fromBytes(value));
  }

  get user(): Bytes {
    let value = this.get("user");
    return value!.toBytes();
  }

  set user(value: Bytes) {
    this.set("user", Value.fromBytes(value));
  }

  get blockNumber(): BigInt {
    let value = this.get("blockNumber");
    return value!.toBigInt();
  }

  set blockNumber(value: BigInt) {
    this.set("blockNumber", Value.fromBigInt(value));
  }

  get blockTimestamp(): BigInt {
    let value = this.get("blockTimestamp");
    return value!.toBigInt();
  }

  set blockTimestamp(value: BigInt) {
    this.set("blockTimestamp", Value.fromBigInt(value));
  }

  get txHash(): Bytes {
    let value = this.get("txHash");
    return value!.toBytes();
  }

  set txHash(value: Bytes) {
    this.set("txHash", Value.fromBytes(value));
  }

  get eventType(): string {
    let value = this.get("eventType");
    return value!.toString();
  }

  set eventType(value: string) {
    this.set("eventType", Value.fromString(value));
  }

  get tokensReturned(): BigInt {
    let value = this.get("tokensReturned");
    return value!.toBigInt();
  }

  set tokensReturned(value: BigInt) {
    this.set("tokensReturned", Value.fromBigInt(value));
  }
}

export class UserSubscriptionRenewalEvent extends Entity {
  constructor(id: Bytes) {
    super();
    this.set("id", Value.fromBytes(id));
  }

  save(): void {
    let id = this.get("id");
    assert(
      id != null,
      "Cannot save UserSubscriptionRenewalEvent entity without an ID"
    );
    if (id) {
      assert(
        id.kind == ValueKind.BYTES,
        `Entities of type UserSubscriptionRenewalEvent must have an ID of type Bytes but the id '${id.displayData()}' is of type ${id.displayKind()}`
      );
      store.set(
        "UserSubscriptionRenewalEvent",
        id.toBytes().toHexString(),
        this
      );
    }
  }

  static load(id: Bytes): UserSubscriptionRenewalEvent | null {
    return changetype<UserSubscriptionRenewalEvent | null>(
      store.get("UserSubscriptionRenewalEvent", id.toHexString())
    );
  }

  get id(): Bytes {
    let value = this.get("id");
    return value!.toBytes();
  }

  set id(value: Bytes) {
    this.set("id", Value.fromBytes(value));
  }

  get user(): Bytes {
    let value = this.get("user");
    return value!.toBytes();
  }

  set user(value: Bytes) {
    this.set("user", Value.fromBytes(value));
  }

  get blockNumber(): BigInt {
    let value = this.get("blockNumber");
    return value!.toBigInt();
  }

  set blockNumber(value: BigInt) {
    this.set("blockNumber", Value.fromBigInt(value));
  }

  get blockTimestamp(): BigInt {
    let value = this.get("blockTimestamp");
    return value!.toBigInt();
  }

  set blockTimestamp(value: BigInt) {
    this.set("blockTimestamp", Value.fromBigInt(value));
  }

  get txHash(): Bytes {
    let value = this.get("txHash");
    return value!.toBytes();
  }

  set txHash(value: Bytes) {
    this.set("txHash", Value.fromBytes(value));
  }

  get eventType(): string {
    let value = this.get("eventType");
    return value!.toString();
  }

  set eventType(value: string) {
    this.set("eventType", Value.fromString(value));
  }

  get currentSubscriptionStart(): BigInt {
    let value = this.get("currentSubscriptionStart");
    return value!.toBigInt();
  }

  set currentSubscriptionStart(value: BigInt) {
    this.set("currentSubscriptionStart", Value.fromBigInt(value));
  }

  get currentSubscriptionEnd(): BigInt {
    let value = this.get("currentSubscriptionEnd");
    return value!.toBigInt();
  }

  set currentSubscriptionEnd(value: BigInt) {
    this.set("currentSubscriptionEnd", Value.fromBigInt(value));
  }

  get currentSubscriptionRate(): BigInt {
    let value = this.get("currentSubscriptionRate");
    return value!.toBigInt();
  }

  set currentSubscriptionRate(value: BigInt) {
    this.set("currentSubscriptionRate", Value.fromBigInt(value));
  }
}

export class UserSubscriptionUpgradeEvent extends Entity {
  constructor(id: Bytes) {
    super();
    this.set("id", Value.fromBytes(id));
  }

  save(): void {
    let id = this.get("id");
    assert(
      id != null,
      "Cannot save UserSubscriptionUpgradeEvent entity without an ID"
    );
    if (id) {
      assert(
        id.kind == ValueKind.BYTES,
        `Entities of type UserSubscriptionUpgradeEvent must have an ID of type Bytes but the id '${id.displayData()}' is of type ${id.displayKind()}`
      );
      store.set(
        "UserSubscriptionUpgradeEvent",
        id.toBytes().toHexString(),
        this
      );
    }
  }

  static load(id: Bytes): UserSubscriptionUpgradeEvent | null {
    return changetype<UserSubscriptionUpgradeEvent | null>(
      store.get("UserSubscriptionUpgradeEvent", id.toHexString())
    );
  }

  get id(): Bytes {
    let value = this.get("id");
    return value!.toBytes();
  }

  set id(value: Bytes) {
    this.set("id", Value.fromBytes(value));
  }

  get user(): Bytes {
    let value = this.get("user");
    return value!.toBytes();
  }

  set user(value: Bytes) {
    this.set("user", Value.fromBytes(value));
  }

  get blockNumber(): BigInt {
    let value = this.get("blockNumber");
    return value!.toBigInt();
  }

  set blockNumber(value: BigInt) {
    this.set("blockNumber", Value.fromBigInt(value));
  }

  get blockTimestamp(): BigInt {
    let value = this.get("blockTimestamp");
    return value!.toBigInt();
  }

  set blockTimestamp(value: BigInt) {
    this.set("blockTimestamp", Value.fromBigInt(value));
  }

  get txHash(): Bytes {
    let value = this.get("txHash");
    return value!.toBytes();
  }

  set txHash(value: Bytes) {
    this.set("txHash", Value.fromBytes(value));
  }

  get eventType(): string {
    let value = this.get("eventType");
    return value!.toString();
  }

  set eventType(value: string) {
    this.set("eventType", Value.fromString(value));
  }

  get previousSubscriptionStart(): BigInt {
    let value = this.get("previousSubscriptionStart");
    return value!.toBigInt();
  }

  set previousSubscriptionStart(value: BigInt) {
    this.set("previousSubscriptionStart", Value.fromBigInt(value));
  }

  get previousSubscriptionEnd(): BigInt {
    let value = this.get("previousSubscriptionEnd");
    return value!.toBigInt();
  }

  set previousSubscriptionEnd(value: BigInt) {
    this.set("previousSubscriptionEnd", Value.fromBigInt(value));
  }

  get previousSubscriptionRate(): BigInt {
    let value = this.get("previousSubscriptionRate");
    return value!.toBigInt();
  }

  set previousSubscriptionRate(value: BigInt) {
    this.set("previousSubscriptionRate", Value.fromBigInt(value));
  }

  get currentSubscriptionStart(): BigInt {
    let value = this.get("currentSubscriptionStart");
    return value!.toBigInt();
  }

  set currentSubscriptionStart(value: BigInt) {
    this.set("currentSubscriptionStart", Value.fromBigInt(value));
  }

  get currentSubscriptionEnd(): BigInt {
    let value = this.get("currentSubscriptionEnd");
    return value!.toBigInt();
  }

  set currentSubscriptionEnd(value: BigInt) {
    this.set("currentSubscriptionEnd", Value.fromBigInt(value));
  }

  get currentSubscriptionRate(): BigInt {
    let value = this.get("currentSubscriptionRate");
    return value!.toBigInt();
  }

  set currentSubscriptionRate(value: BigInt) {
    this.set("currentSubscriptionRate", Value.fromBigInt(value));
  }
}

export class UserSubscriptionDowngradeEvent extends Entity {
  constructor(id: Bytes) {
    super();
    this.set("id", Value.fromBytes(id));
  }

  save(): void {
    let id = this.get("id");
    assert(
      id != null,
      "Cannot save UserSubscriptionDowngradeEvent entity without an ID"
    );
    if (id) {
      assert(
        id.kind == ValueKind.BYTES,
        `Entities of type UserSubscriptionDowngradeEvent must have an ID of type Bytes but the id '${id.displayData()}' is of type ${id.displayKind()}`
      );
      store.set(
        "UserSubscriptionDowngradeEvent",
        id.toBytes().toHexString(),
        this
      );
    }
  }

  static load(id: Bytes): UserSubscriptionDowngradeEvent | null {
    return changetype<UserSubscriptionDowngradeEvent | null>(
      store.get("UserSubscriptionDowngradeEvent", id.toHexString())
    );
  }

  get id(): Bytes {
    let value = this.get("id");
    return value!.toBytes();
  }

  set id(value: Bytes) {
    this.set("id", Value.fromBytes(value));
  }

  get user(): Bytes {
    let value = this.get("user");
    return value!.toBytes();
  }

  set user(value: Bytes) {
    this.set("user", Value.fromBytes(value));
  }

  get blockNumber(): BigInt {
    let value = this.get("blockNumber");
    return value!.toBigInt();
  }

  set blockNumber(value: BigInt) {
    this.set("blockNumber", Value.fromBigInt(value));
  }

  get blockTimestamp(): BigInt {
    let value = this.get("blockTimestamp");
    return value!.toBigInt();
  }

  set blockTimestamp(value: BigInt) {
    this.set("blockTimestamp", Value.fromBigInt(value));
  }

  get txHash(): Bytes {
    let value = this.get("txHash");
    return value!.toBytes();
  }

  set txHash(value: Bytes) {
    this.set("txHash", Value.fromBytes(value));
  }

  get eventType(): string {
    let value = this.get("eventType");
    return value!.toString();
  }

  set eventType(value: string) {
    this.set("eventType", Value.fromString(value));
  }

  get previousSubscriptionStart(): BigInt {
    let value = this.get("previousSubscriptionStart");
    return value!.toBigInt();
  }

  set previousSubscriptionStart(value: BigInt) {
    this.set("previousSubscriptionStart", Value.fromBigInt(value));
  }

  get previousSubscriptionEnd(): BigInt {
    let value = this.get("previousSubscriptionEnd");
    return value!.toBigInt();
  }

  set previousSubscriptionEnd(value: BigInt) {
    this.set("previousSubscriptionEnd", Value.fromBigInt(value));
  }

  get previousSubscriptionRate(): BigInt {
    let value = this.get("previousSubscriptionRate");
    return value!.toBigInt();
  }

  set previousSubscriptionRate(value: BigInt) {
    this.set("previousSubscriptionRate", Value.fromBigInt(value));
  }

  get currentSubscriptionStart(): BigInt {
    let value = this.get("currentSubscriptionStart");
    return value!.toBigInt();
  }

  set currentSubscriptionStart(value: BigInt) {
    this.set("currentSubscriptionStart", Value.fromBigInt(value));
  }

  get currentSubscriptionEnd(): BigInt {
    let value = this.get("currentSubscriptionEnd");
    return value!.toBigInt();
  }

  set currentSubscriptionEnd(value: BigInt) {
    this.set("currentSubscriptionEnd", Value.fromBigInt(value));
  }

  get currentSubscriptionRate(): BigInt {
    let value = this.get("currentSubscriptionRate");
    return value!.toBigInt();
  }

  set currentSubscriptionRate(value: BigInt) {
    this.set("currentSubscriptionRate", Value.fromBigInt(value));
  }
}
