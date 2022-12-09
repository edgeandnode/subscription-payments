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

  get subscriber(): Bytes {
    let value = this.get("subscriber");
    return value!.toBytes();
  }

  set subscriber(value: Bytes) {
    this.set("subscriber", Value.fromBytes(value));
  }

  get startBlock(): BigInt {
    let value = this.get("startBlock");
    return value!.toBigInt();
  }

  set startBlock(value: BigInt) {
    this.set("startBlock", Value.fromBigInt(value));
  }

  get endBlock(): BigInt {
    let value = this.get("endBlock");
    return value!.toBigInt();
  }

  set endBlock(value: BigInt) {
    this.set("endBlock", Value.fromBigInt(value));
  }

  get pricePerBlock(): BigInt {
    let value = this.get("pricePerBlock");
    return value!.toBigInt();
  }

  set pricePerBlock(value: BigInt) {
    this.set("pricePerBlock", Value.fromBigInt(value));
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

  get subscriber(): Bytes {
    let value = this.get("subscriber");
    return value!.toBytes();
  }

  set subscriber(value: Bytes) {
    this.set("subscriber", Value.fromBytes(value));
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

  get subscriber(): Bytes {
    let value = this.get("subscriber");
    return value!.toBytes();
  }

  set subscriber(value: Bytes) {
    this.set("subscriber", Value.fromBytes(value));
  }

  get startBlock(): BigInt {
    let value = this.get("startBlock");
    return value!.toBigInt();
  }

  set startBlock(value: BigInt) {
    this.set("startBlock", Value.fromBigInt(value));
  }

  get endBlock(): BigInt {
    let value = this.get("endBlock");
    return value!.toBigInt();
  }

  set endBlock(value: BigInt) {
    this.set("endBlock", Value.fromBigInt(value));
  }

  get pricePerBlock(): BigInt {
    let value = this.get("pricePerBlock");
    return value!.toBigInt();
  }

  set pricePerBlock(value: BigInt) {
    this.set("pricePerBlock", Value.fromBigInt(value));
  }
}
