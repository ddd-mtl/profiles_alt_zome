mod validate;

use hdi::prelude::*;


///-------------------------------------------------------------------------------------------------
/// Global consts
///-------------------------------------------------------------------------------------------------

/// DNA/Zome names
pub const DEFAULT_COORDINATOR_ZOME_NAME: &'static str = "zSharedOwnership";
pub const DEFAULT_INTEGRITY_ZOME_NAME: &'static str = "shared_ownership_integrity";

/// ANCHOR NAMES
pub const ROOT_ANCHOR_SHAREDS: &'static str = "all_shareds";


///-------------------------------------------------------------------------------------------------
/// Entry types
///-------------------------------------------------------------------------------------------------

#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct SharedKey {
    pub key: String,
}


#[hdk_entry_types]
#[unit_enum(SharedOwnershipEntryTypes)]
pub enum SharedOwnershipEntry {
    SharedKey(SharedKey),
}


///-------------------------------------------------------------------------------------------------
/// Link types
///-------------------------------------------------------------------------------------------------

#[hdk_link_types]
#[derive(Serialize, Deserialize)]
pub enum SharedOwnershipLinkType {
    SharedPath,
    SharedEntry,
    Shared,
    Owner,
}


/// Tag data used for validation
#[derive(Debug, Clone, Serialize, Deserialize, SerializedBytes)]
pub struct TagShared {
    pub signature: Signature,
    pub maybe_owner_link_ah: Option<ActionHash>,
}

/// Tag data used for validation
#[derive(Debug, Clone, Serialize, Deserialize, SerializedBytes)]
pub struct TagOwner {
    pub shared_link_ah: ActionHash,
}