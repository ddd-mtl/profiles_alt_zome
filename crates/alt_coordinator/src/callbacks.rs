use hdk::prelude::*;
use zome_utils::*;
use zome_signals::*;
use hc_zome_profiles_integrity::*;

#[hdk_extern(infallible)]
pub fn post_commit(signedActionList: Vec<SignedActionHashed>) {
   debug!("ProfilesAlt post_commit() called for {} actions. ({})", signedActionList.len(), zome_info().unwrap().id);
   std::panic::set_hook(Box::new(zome_panic_hook));
   emit_post_commit::<EntryTypes, LinkTypes>(signedActionList);
}

