// SPDX-License-Identifier: MIT
pragma solidity 0.8.19;

import "@openzeppelin/contracts/access/Ownable.sol";

/// @notice Data for an entry, a set of subscriptions contracts and a metadata hash.
struct Entry {
    address[] subscriptions;
    bytes32 metadataHash;
}

/// @notice This contract is designed to store an allowlist of entries, where each entry is associated with a set of
/// subscriptions contracts and a metadata hash. Entries are added and removed using a unique ID.
/// @dev This contract does not need to be upgradeable (yet), since it serves as a closed allowlist for the
/// Graph Explorer (for now). Therefore, a new contract can be easily transitioned to.
contract Registry is Ownable {
    event InsertEntry(uint256 indexed id, Entry entry);
    event RemoveEntry(uint256 indexed id);

    /// @notice Mapping of entry ID to its associated data.
    mapping(uint256 => Entry) public entries;

    /// @notice Insert the given entry data, associated with the given `_id`.
    function insertEntry(uint256 _id, Entry calldata _entry) public onlyOwner {
        entries[_id] = _entry;
        emit InsertEntry(_id, _entry);
    }

    /// @notice Remove the entry data associated with the given `_id`.
    function removeEntry(uint256 _id) public onlyOwner {
        delete entries[_id];
        emit RemoveEntry(_id);
    }

    /// @notice Return the entry data, (subscriptions, metadataHash), associated with the given `_id`.
    function getEntry(uint256 _id) external view returns (address[] memory, bytes32) {
        Entry memory _entry = entries[_id];
        return (_entry.subscriptions, _entry.metadataHash);
    }

    // TODO: Add ability to transfer subscriptions, will require audit.
}
