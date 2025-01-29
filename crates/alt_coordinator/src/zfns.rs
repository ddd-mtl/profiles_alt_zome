use hdk::prelude::*;
use zome_utils::*;
use zome_signals::*;
use hc_zome_profiles_integrity::*;


/// (zits currently cant handle destructured function arguments)
#[hdk_extern]
#[feature(zits_blocking)]
pub fn create_profile(pair: (Profile, AgentPubKey)) -> ExternResult<ActionHash> {
   let profile = pair.0;
   let agent_address = pair.1;
   std::panic::set_hook(Box::new(zome_panic_hook));
   /// Create Entry
   let ah = create_entry(EntryTypes::Profile(profile.clone()))?;
   /// Create Links
   let path = prefix_path(profile.nickname.clone())?;
   path.ensure()?;
   let _ = create_link(
      path.path_entry_hash()?,
      agent_address.clone(),
      LinkTypes::PathToAgent,
      LinkTag::new(profile.nickname.to_lowercase().as_bytes().to_vec()),
   )?;
   let _ = create_link(
      agent_address,
      ah.clone(),
      LinkTypes::AgentToProfile,
      (),
   )?;
   /// Done
   Ok(ah)
}


///
#[hdk_extern]
#[feature(zits_blocking)]
pub fn update_profile(pair: (Profile, AgentPubKey)) -> ExternResult<ActionHash> {
   let profile = pair.0;
   let agent_address = pair.1;
   std::panic::set_hook(Box::new(zome_panic_hook));
   /// Update Entry
   let Some((previous_profile, previous_record, previous_link)) = find_latest_profile(agent_address.clone())?
      else { return zome_error!("No profile to update"); };
   let new_ah = update_entry(previous_record.action_address().to_owned(), &profile)?;
   /// "Update" link
   let _ = delete_link(previous_link.create_link_hash)?;
   let _ = create_link(
      agent_address.clone(),
      new_ah.clone(),
      LinkTypes::AgentToProfile,
      (),
   )?;
   /// If we have changed the nickname, remove the previous nickname link and add a new one
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
      let _ = create_link(
         path.path_entry_hash()?,
         agent_address,
         LinkTypes::PathToAgent,
         LinkTag::new(profile.nickname.to_lowercase().as_bytes().to_vec()),
      )?;
   }
   ///
   Ok(new_ah)
}


/// From a nickname filter of at least 3 characters, returns all the agents whose nickname starts with that prefix
/// Ignores the nickname case, will return upper or lower case nicknames that match
#[hdk_extern]
pub fn search_agents(nickname_filter: String) -> ExternResult<Vec<AgentPubKey>> {
   std::panic::set_hook(Box::new(zome_panic_hook));
   if nickname_filter.len() < 3 {
      return zome_error!("Cannot search with a prefix less than 3 characters");
   }
   ///
   let prefix_path = prefix_path(nickname_filter.clone())?;
   let input = GetLinksInputBuilder::try_new(AnyLinkableHash::from(prefix_path.path_entry_hash()?), LinkTypes::PathToAgent)?
       .tag_prefix(LinkTag::new(nickname_filter.to_lowercase().as_bytes().to_vec())).build();
   let links = get_links(input)?;
   ///
   let mut agents: Vec<AgentPubKey> = vec![];
   for link in links {
      if let Ok(pub_key) = AgentPubKey::try_from(link.target) {
         agents.push(pub_key);
      }
   }
   ///
   Ok(agents)
}

/// Return the profile for the given agent, if any
#[hdk_extern]
pub fn find_profile(agent_pub_key: AgentPubKey) -> ExternResult<Option<(ActionHash, Profile)>> {
   std::panic::set_hook(Box::new(zome_panic_hook));
   let Some((profile, record, link)) = find_latest_profile(agent_pub_key)?
      else { return Ok(None) };
   ///
   emit_link_signal(link, StateChange::Create(false))?;
   emit_new_entry_signal(record.clone(), false)?;
   ///
   Ok(Some((record.action_address().to_owned(), profile)))
}


/// Return the latest profile for the given agent, if any
pub fn find_latest_profile(agent_pub_key: AgentPubKey) -> ExternResult<Option<(Profile, Record, Link)>> {
   let links = get_links(GetLinksInputBuilder::try_new(agent_pub_key, LinkTypes::AgentToProfile)?.build())?;
   if links.len() == 0 {
      return Ok(None);
   }
   if links.len() > 1 {
      warn!("Multiple profiles found for an agent. Taking first one");
   }
   let link = &links[0];
   let first_profile_ah = link.target.clone().into_action_hash().unwrap();
   let record = get_latest_record(first_profile_ah.clone())?;
   let profile = get_typed_from_record::<Profile>(record.clone())?;
   ///
   Ok(Some((profile, record, link.clone())))
}



/// Gets all the agents that have created a profile in this DHT.
#[hdk_extern]
#[feature(zits_blocking)]
pub fn probe_profiles(_: ()) -> ExternResult<()> {
   std::panic::set_hook(Box::new(zome_panic_hook));
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
   for link in &links {
      if let Ok(pub_key) = AgentPubKey::try_from(link.target.to_owned()) {
         agents.push(pub_key);
      }
   }
   ///
   emit_links_signal(links)?;
   ///
   Ok(())
}


///
pub fn prefix_path(nickname: String) -> ExternResult<TypedPath> {
   // convert to lowercase for path for ease of search
   let lower_nickname = nickname.to_lowercase();
   let prefix: String = lower_nickname.chars().take(3).collect();
   return Path::from(format!("all_profiles.{}", prefix)).typed(LinkTypes::PrefixPath);
}
