//use std::collections::BTreeMap;
use hdi::prelude::*;


#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct MiniProfile {
    pub nickname: String,
    //pub fields: BTreeMap<String, String>,
}


#[hdk_entry_defs]
#[unit_enum(MiniProfilesEntryTypes)]
pub enum MiniProfilesEntry {
    MiniProfile(MiniProfile),
}


#[hdk_link_types]
#[derive(Serialize, Deserialize)]
pub enum MiniProfilesLinkType {
    PrefixPath,
    PathToAgent,
    AgentToProfile,
}
