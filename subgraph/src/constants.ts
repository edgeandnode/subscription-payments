import {BigInt} from '@graphprotocol/graph-ts';

export const USER_SUBSCRIPTION_EVENT_TYPE__CREATED = 'CREATED';
export const USER_SUBSCRIPTION_EVENT_TYPE__CANCELED = 'CANCELED';
export const USER_SUBSCRIPTION_EVENT_TYPE__RENEW = 'RENEW';
export const USER_SUBSCRIPTION_EVENT_TYPE__UPGRADE = 'UPGRADE';
export const USER_SUBSCRIPTION_EVENT_TYPE__DOWNGRADE = 'DOWNGRADE';

export const ONE_MINUTE = 60;
export const ONE_HOUR = ONE_MINUTE * 60;
export const ONE_DAY = ONE_HOUR * 24;
export const BILLING_PERIOD_SECONDS = ONE_DAY * 30;
export const BILLING_PERIOD_SECONDS_BIGINT = BigInt.fromI64(
  BILLING_PERIOD_SECONDS
);
