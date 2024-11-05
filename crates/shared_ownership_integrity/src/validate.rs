use hdi::prelude::*;
use crate::entry_types::validate_app_entry;

///
#[hdk_extern]
fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
   //debug!("*** validate() op = {:?}", op);
   match op {
      Op::StoreRecord ( _ ) => Ok(ValidateCallbackResult::Valid),
      Op::StoreEntry(storeEntry) => {
         let creation_action = storeEntry.action.hashed.into_inner().0;
         return validate_create_entry(creation_action.clone(), storeEntry.entry);
      },
      Op::RegisterCreateLink(registered_create_link) => {
         let (create, signature) = registered_create_link.create_link.into_inner();
         return validate_create_link(create, signature);
      },
      Op::RegisterDeleteLink (_)=> Ok(ValidateCallbackResult::Valid),
      Op::RegisterUpdate { .. } => Ok(ValidateCallbackResult::Valid),
      Op::RegisterDelete { .. } => Ok(ValidateCallbackResult::Valid),
      Op::RegisterAgentActivity { .. } => Ok(ValidateCallbackResult::Valid),
   }
}


/// Dispatch according to base type
fn validate_create_entry(creation_action: EntryCreationAction, entry: Entry) -> ExternResult<ValidateCallbackResult> {
   let result = match entry.clone() {
      Entry::CounterSign(_data, _bytes) => Ok(ValidateCallbackResult::Invalid("CounterSign not allowed".into())),
      Entry::Agent(_agent_key) => Ok(ValidateCallbackResult::Valid),
      Entry::CapClaim(_claim) => Ok(ValidateCallbackResult::Valid),
      Entry::CapGrant(_grant) => Ok(ValidateCallbackResult::Valid),
      Entry::App(entry_bytes) => {
         let EntryType::App(app_entry_def) = creation_action.entry_type().clone()
            else { unreachable!() };
         let shared_key = SharedKey::try_from(entry_bytes)?;
         Ok(ValidateCallbackResult::Valid)
      },
   };
   /// Done
   //debug!("*** validate_create_entry() result = {:?}", result);
   result
}


///
fn validate_create_link(create_link: HoloHashed<CreateLink>, _signature: Signature) -> ExternResult<ValidateCallbackResult>  {
   // debug!("validate_create_link(): {:?}", create_link);
   match create_link.link_type {
      SharedOwnershipEntryLinkType::Shared => {
         /// Convert tag
         let tag_bytes = create_link.tag.clone().into_inner();
         let unsafe_bytes = UnsafeBytes::from(tag_bytes.clone());
         let ser_bytes = SerializedBytes::from(unsafe_bytes);
         let tag_shared: TagShared = TagShared::try_from(ser_bytes)?;
         // FIXME: check signature is base's signing of target
         /// Check link author is an owner
         let owner: AgentPubKey = create_link.author.into();
         let is_owner = is_owner_from_owner_link(owner, create_link.target.into(), tag_shared.maybe_owner_link_ah)?;
         if (!is_owner) {
            return Ok(ValidateCallbackResult::Failed)
         }
         Ok(ValidateCallbackResult::Valid)
      },
      SharedOwnershipEntryLinkType::Owner => {
         /// Convert tag
         let tag_bytes = create_link.tag.clone().into_inner();
         let unsafe_bytes = UnsafeBytes::from(tag_bytes.clone());
         let ser_bytes = SerializedBytes::from(unsafe_bytes);
         let tag_owner: TagOwner = TagOwner::try_from(ser_bytes)?;
         /// Check link target is an owner
         let new_owner: AgentPubKey = create_link.target.into();
         let is_owner = is_owner_from_owner_link(new_owner, create_link.base.into(), tag_owner.shared_link_ah)?;
         if (!is_owner) {
            return Ok(ValidateCallbackResult::Failed)
         }
         Ok(ValidateCallbackResult::Valid)
      },
      SharedOwnershipEntryLinkType::PrefixPath => {
         Ok(ValidateCallbackResult::Valid)
      },
      _ => panic!("Unknown link type"),
   }
}

///
fn is_owner_from_shared_link(agent: AgentPubKey, shared_ah: ActionHash, maybe_owner_link_ah: Option<ActionHash>) -> ExternResult<bool> {
   let shared_record = must_get_valid_record(shared_ah)?;
   if shared_record.action().author == agent {
      return Ok(true);
   }
   let Some(owner_link_ah) = maybe_owner_link_ah
      else { return Ok(false) };
   let link_record = must_get_valid_record(owner_link_ah)?;
   // FIXME: convert to CreateLink
   // FIXME: Check its a owner link from agent to shared_ah
   // FIXME: call is_owner_from_owner_link();
   /// Done
   Ok(true)
}


///
fn is_owner_from_owner_link(agent: AgentPubKey, shared_ah: ActionHash, shared_link_ah: ActionHash) -> ExternResult<bool> {
   let shared_record = must_get_valid_record(shared_ah)?;
   if shared_record.action().author == agent {
      return Ok(true);
   }
   let link_record = must_get_valid_record(link_ah)?;
   // FIXME: convert to CreateLink
   // FIXME: Check its a Shared link from agent to shared_ah
   /// Done
   Ok(true)
}