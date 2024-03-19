use hdk::prelude::*;
use hc_zome_profiles_integrity::*;
use crate::utils::*;

///
#[hdk_extern]
pub fn create_profile(pair: (Profile, AgentPubKey)) -> ExternResult<Record> {
   let profile = pair.0;
   let agent_address = pair.1;

   let ah = create_entry(EntryTypes::Profile(profile.clone()))?;

   let path = prefix_path(profile.nickname.clone())?;
   path.ensure()?;


   create_link(
      path.path_entry_hash()?,
      agent_address.clone(),
      LinkTypes::PathToAgent,
      LinkTag::new(profile.nickname.to_lowercase().as_bytes().to_vec()),
   )?;
   create_link(
      agent_address,
      ah.clone(),
      LinkTypes::AgentToProfile,
      (),
   )?;

   let record = get(ah, GetOptions::default())?
      .ok_or(wasm_error!(WasmErrorInner::Guest("Unreachable".into())))?;

   Ok(record)
}



///
#[hdk_extern]
pub fn update_profile(pair: (Profile, AgentPubKey)) -> ExternResult<Record> {
   let profile = pair.0;
   let agent_address = pair.1;

   let previous_profile_record = get_profile(agent_address.clone())?
      .ok_or(wasm_error!(WasmErrorInner::Guest(
            "I haven't created a profile yet".into(),
        )))?;

   let action_hash = update_entry(previous_profile_record.action_address().clone(), &profile)?;

   // If we have changed the nickname, remove the previous nickname link and add a new one
   let previous_profile: Profile = previous_profile_record
      .entry()
      .to_app_option()
      .map_err(|e| wasm_error!(e))?
      .ok_or(wasm_error!(WasmErrorInner::Guest(
            "Previous profile is malformed".to_string()
        )))?;
   if previous_profile.nickname.ne(&profile.nickname) {
      let previous_prefix_path = prefix_path(previous_profile.nickname)?;
      let links = get_links(GetLinksInputBuilder::try_new(
         AnyLinkableHash::from(previous_prefix_path.path_entry_hash()?),
         LinkTypes::PathToAgent,
      )?.build())?;

      for l in links {
         if let Ok(pub_key) = AgentPubKey::try_from(l.target) {
            if agent_address.eq(&pub_key) {
               delete_link(l.create_link_hash)?;
            }
         }
      }

      let path = prefix_path(profile.nickname.clone())?;

      path.ensure()?;

      create_link(
         path.path_entry_hash()?,
         agent_address,
         LinkTypes::PathToAgent,
         LinkTag::new(profile.nickname.to_lowercase().as_bytes().to_vec()),
      )?;
   }

   let record = get(action_hash, GetOptions::default())?
      .ok_or(wasm_error!(WasmErrorInner::Guest("Unreachable".into())))?;

   Ok(record)
}


/// From a nickname filter of at least 3 characters, returns all the agents whose nickname starts with that prefix
/// Ignores the nickname case, will return upper or lower case nicknames that match
#[hdk_extern]
pub fn search_agents(nickname_filter: String) -> ExternResult<Vec<AgentPubKey>> {
   if nickname_filter.len() < 3 {
      return Err(wasm_error!(WasmErrorInner::Guest(
            "Cannot search with a prefix less than 3 characters".into(),
        )));
   }

   let prefix_path = prefix_path(nickname_filter.clone())?;
   let input = GetLinksInputBuilder::try_new(AnyLinkableHash::from(prefix_path.path_entry_hash()?), LinkTypes::PathToAgent)?
       .tag_prefix(LinkTag::new(nickname_filter.to_lowercase().as_bytes().to_vec())).build();
   let links = get_links(input)?;

   let mut agents: Vec<AgentPubKey> = vec![];

   for link in links {
      if let Ok(pub_key) = AgentPubKey::try_from(link.target) {
         agents.push(pub_key);
      }
   }

   Ok(agents)
}


/// Returns the profile for the given agent, if they have created it.
#[hdk_extern]
pub fn get_profile(agent_pub_key: AgentPubKey) -> ExternResult<Option<Record>> {
   let links = get_links(GetLinksInputBuilder::try_new(agent_pub_key, LinkTypes::AgentToProfile)?.build())?;
   if links.len() == 0 {
      return Ok(None);
   }

   let link = links[0].clone();

   let profile = get_latest(link.target.into_action_hash().ok_or(wasm_error!(
        WasmErrorInner::Guest("Profile link target is not of ActionHash".into())
    ))?)?;

   Ok(Some(profile))
}


fn get_latest(action_hash: ActionHash) -> ExternResult<Record> {
   let details = get_details(action_hash, GetOptions::default())?.ok_or(wasm_error!(
        WasmErrorInner::Guest("Profile not found".into())
    ))?;

   match details {
      Details::Entry(_) => Err(wasm_error!(WasmErrorInner::Guest(
            "Malformed details".into()
        ))),
      Details::Record(element_details) => match element_details.updates.last() {
         Some(update) => get_latest(update.action_address().clone()),
         None => Ok(element_details.record),
      },
   }
}


/// Gets all the agents that have created a profile in this DHT.
#[hdk_extern]
pub fn get_agents_with_profile(_: ()) -> ExternResult<Vec<AgentPubKey>> {
   let path = Path::from("all_profiles").typed(LinkTypes::PrefixPath)?;

   let children = path.children_paths()?;

   let get_links_input: Vec<GetLinksInput> = children
      .into_iter()
      .map(|path| {
         Ok(GetLinksInputBuilder::try_new(
            AnyLinkableHash::from(path.path_entry_hash()?),
            LinkTypes::PathToAgent.try_into_filter()?
         ).unwrap().build())
      })
      .collect::<ExternResult<Vec<GetLinksInput>>>()?;

   let links = HDK
      .with(|h| h.borrow().get_links(get_links_input))?
      .into_iter()
      .flatten()
      .collect::<Vec<Link>>();

   let mut agents: Vec<AgentPubKey> = vec![];

   for link in links {
      if let Ok(pub_key) = AgentPubKey::try_from(link.target) {
         agents.push(pub_key);
      }
   }

   Ok(agents)
}


