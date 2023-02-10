import {Address, ByteArray, Bytes, crypto} from '@graphprotocol/graph-ts';

/**
 * Generate a keccak256 hex string of the user:authorizedSigner
 * @param subscriptionOwner address of the ActiveSubscription owner
 * @param authorizedSigner address of the user authorized to sign for the owner of the ActiveSubscription
 * @returns Bytes representation of a hex string concatenation of the `user:authorizedSigner` to create a unique id
 */
export function buildAuthorizedSignerId(
  subscriptionOwner: Address,
  authorizedSigner: Address
): Bytes {
  let hash = crypto.keccak256(
    ByteArray.fromUTF8(`${subscriptionOwner}:${authorizedSigner}`)
  );
  return Bytes.fromByteArray(hash);
}
