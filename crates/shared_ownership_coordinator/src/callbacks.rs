use hdk::prelude::*;
use zome_utils::*;
use zome_signals::*;
use shared_ownership_integrity::*;


#[hdk_extern]
pub fn init(_: ()) -> ExternResult<InitCallbackResult> {
   debug!("SharedOwnership.init() CALLED");
   let mut fns = BTreeSet::new();
   fns.insert((zome_info()?.name, FunctionName("recv_remote_signal".into())));
   fns.insert((zome_info()?.name, FunctionName("recv_shared_key".into())));
   let cap_grant_entry: CapGrantEntry = CapGrantEntry::new(
      String::from("remote signals"), // A string by which to later query for saved grants.
      CapAccess::Unrestricted, // Unrestricted access means any external agent can call the extern
      GrantedFunctions::Listed(fns),
   );
   create_cap_grant(cap_grant_entry)?;
   /// Done
   Ok(InitCallbackResult::Pass)
}


///
#[hdk_extern(infallible)]
pub fn post_commit(signedActionList: Vec<SignedActionHashed>) {
   //debug!("SharedOwnership.post_commit() called for {} actions. ({})", signedActionList.len(), zome_info().unwrap().id);
   std::panic::set_hook(Box::new(zome_panic_hook));
   attest_post_commit::<SharedOwnershipEntry, SharedOwnershipLinkType>(signedActionList);
}

