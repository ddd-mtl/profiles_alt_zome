use hdi::prelude::*;
use crate::*;

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
      Entry::App(_entry_bytes) => {
         let EntryType::App(_app_entry_def) = creation_action.entry_type().clone()
            else { unreachable!() };
         let _shared_key = SharedKey::try_from(entry)?;
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
   let typed_link_type = SharedOwnershipEntryLinkType::from_type(create_link.zome_index, create_link.link_type)?.unwrap();
   match typed_link_type {
      SharedOwnershipEntryLinkType::Shared => {
         /// Convert tag
         let tag_bytes = create_link.tag.clone().into_inner();
         let unsafe_bytes = UnsafeBytes::from(tag_bytes.clone());
         let ser_bytes = SerializedBytes::from(unsafe_bytes);
         let tag_shared: TagShared = TagShared::try_from(ser_bytes).unwrap();
         // FIXME: check signature is base's signing of target
         /// Check link author is an owner
         let owner: AgentPubKey = create_link.content.author.clone().into();
         let Some(shared_ah) = create_link.content.target_address.into_action_hash()
            else {return Ok(ValidateCallbackResult::Invalid("Link target is not an ActionHash".to_string()))};
         let is_owner = is_owner_from_shared_link(&owner, shared_ah, tag_shared.maybe_owner_link_ah)?;
         if !is_owner {
            return Ok(ValidateCallbackResult::Invalid("Link author is not an onwer".to_string()))
         }
         Ok(ValidateCallbackResult::Valid)
      },
      SharedOwnershipEntryLinkType::Owner => {
         /// Convert tag
         let tag_bytes = create_link.tag.clone().into_inner();
         let unsafe_bytes = UnsafeBytes::from(tag_bytes.clone());
         let ser_bytes = SerializedBytes::from(unsafe_bytes);
         let tag_owner: TagOwner = TagOwner::try_from(ser_bytes).unwrap();
         /// Check link target is an owner
         let Some(new_owner) = create_link.content.target_address.clone().into_agent_pub_key()
            else {return Ok(ValidateCallbackResult::Invalid("Link target is not an AgentPubKey".to_string()))};
         let Some(shared_ah) = create_link.content.base_address.into_action_hash()
            else {return Ok(ValidateCallbackResult::Invalid("Link base is not an ActionHash".to_string()))};
         let is_owner = is_owner_from_owner_link(&new_owner, shared_ah, tag_owner.shared_link_ah)?;
         if !is_owner {
            return Ok(ValidateCallbackResult::Invalid("Link target is not an owner".to_string()))
         }
         Ok(ValidateCallbackResult::Valid)
      },
      SharedOwnershipEntryLinkType::PrefixPath => {
         Ok(ValidateCallbackResult::Valid)
      },
      //_ => panic!("Unknown link type"),
   }
}


///
fn is_owner_from_shared_link(agent: &AgentPubKey, shared_ah: ActionHash, maybe_owner_link_ah: Option<ActionHash>) -> ExternResult<bool> {
   let shared_record = must_get_valid_record(shared_ah.clone())?;
   /// Ok if agent is shared's author
   if shared_record.action().author() == agent {
      return Ok(true);
   }
   /// Get Owner link CreateLink action
   let Some(owner_link_ah) = maybe_owner_link_ah
      else { return Ok(false) };
   let link_record = must_get_valid_record(owner_link_ah)?;
   let Action::CreateLink(create_link) = link_record.action()
      else { return  Err(wasm_error!("Record does not hold a CreateLink")); };
   let Some(target_agent) = create_link.target_address.clone().into_agent_pub_key()
      else {return Ok(false)};
   let Some(typed_link_type) = SharedOwnershipEntryLinkType::from_type(create_link.zome_index, create_link.link_type)?
      else {return Ok(false)};
   /// Check if it's an Owner link from shared_ah to agent
   let is_valid_owner_link =
       typed_link_type == SharedOwnershipEntryLinkType::Owner
       && &target_agent == agent
       && create_link.base_address == shared_ah.clone().into();
   if !is_valid_owner_link {
      return Ok(false)
   }
   /// Check if the Shared link is valid
   let tag_bytes = create_link.tag.clone().into_inner();
   let unsafe_bytes = UnsafeBytes::from(tag_bytes.clone());
   let ser_bytes = SerializedBytes::from(unsafe_bytes);
   let tag_owner: TagOwner = TagOwner::try_from(ser_bytes).unwrap();
   let is_valid_shared_link = is_owner_from_owner_link(agent, shared_ah.clone(), tag_owner.shared_link_ah)?;
   /// Done
   Ok(is_valid_shared_link)
}


///
fn is_owner_from_owner_link(agent: &AgentPubKey, shared_ah: ActionHash, shared_link_ah: ActionHash) -> ExternResult<bool> {
   let shared_record = must_get_valid_record(shared_ah.clone())?;
   /// Ok if agent is shared's author
   if shared_record.action().author() == agent {
      return Ok(true);
   }
   /// convert to CreateLink
   let link_record = must_get_valid_record(shared_link_ah)?;
   let Action::CreateLink(create_link) = link_record.action()
      else { return  Err(wasm_error!("Record does not hold a CreateLink")); };
   let Some(base_agent) = create_link.base_address.clone().into_agent_pub_key()
      else {return Ok(false)};
   let Some(typed_link_type) = SharedOwnershipEntryLinkType::from_type(create_link.zome_index, create_link.link_type)?
      else {return Ok(false)};
   /// Check it's a Shared link from agent to shared_ah
   let is_valid =
     typed_link_type == SharedOwnershipEntryLinkType::Shared
   && &base_agent == agent
   && create_link.target_address == shared_ah.clone().into();
   /// Done
   Ok(is_valid)
}