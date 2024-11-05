mod validate;

use hdi::prelude::*;


///-------------------------------------------------------------------------------------------------
/// Global consts
///-------------------------------------------------------------------------------------------------

/// DNA/Zome names
pub const VINES_DEFAULT_ROLE_NAME: &'static str = "rVines";
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


#[hdk_entry_defs]
#[unit_enum(SharedOwnershipEntryTypes)]
pub enum SharedOwnershipEntry {
    SharedKey(SharedKey),
}


///-------------------------------------------------------------------------------------------------
/// Link types
///-------------------------------------------------------------------------------------------------

#[hdk_link_types]
#[derive(Serialize, Deserialize)]
pub enum SharedOwnershipEntryLinkType {
    PrefixPath,
    Shared,
    Owner,
}


/// Tag data used for validation
pub struct TagShared {
    signature: SerializedBytes,
    maybe_owner_link_ah: Option<ActionHash>,
}

/// Tag data used for validation
pub struct TagOwner {
    shared_link_ah: ActionHash,
}