use hdk::prelude::*;
use zome_utils::*;
use zome_signals::*;
use shared_ownership_integrity::*;


///
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishOwnershipInput {
  pub shared_ah: ActionHash,
  pub non_author: Option<(AgentPubKey, Signature)>, // if no signature then caller must be author
}


/// Return (SharedLinkAh, OwnerLinkAh)
#[hdk_extern]
#[feature(zits_blocking)]
pub fn publish_ownership(input: PublishOwnershipInput) -> ExternResult<(ActionHash, ActionHash)> {
  std::panic::set_hook(Box::new(zome_panic_hook));
  /// Get owner proof
  let mut maybe_owner_link_ah = None;
  let (agent, signature) = if let Some((agent, signed)) = input.non_author.clone() {
    let owners = probe_owners(input.shared_ah.clone())?;
    let Some(pair) = owners.iter().filter(|&(owner, _link_ah)| owner == &agent).next()
      else { return zome_error!("Agent is not an owner of shared entry"); };
    maybe_owner_link_ah = Some(pair.clone().1);
    (agent, Some(signed.clone()))
  } else {
    /// Sign it
    let signed = sign(agent_info()?.agent_initial_pubkey, input.shared_ah.clone())?;
    (agent_info()?.agent_initial_pubkey, Some(signed))
  };
  /// Create tag
  let tag: TagShared = TagShared {
    signature: signature.unwrap(),
    maybe_owner_link_ah,
  };
  /// Create Shared link
  let shared_link_ah = create_link(agent.clone(), input.shared_ah.clone(), SharedOwnershipLinkType::Shared, obj2Tag(tag)?)?;
  /// Create OwnerLink
  let tag: TagOwner = TagOwner { shared_link_ah: shared_link_ah.clone() };
  let owner_link_ah = create_link(input.shared_ah.clone(), agent, SharedOwnershipLinkType::Owner, obj2Tag(tag)?)?;
  /// Create SharedPath Link
  if input.non_author.is_none() {
    let tp = Path::from(ROOT_ANCHOR_SHAREDS).typed(SharedOwnershipLinkType::SharedPath)?;
    tp.ensure()?;
    let ph = tp.path_entry_hash()?;
    create_link(
      ph,
      input.shared_ah.clone(),
      SharedOwnershipLinkType::SharedEntry,
      LinkTag::new(vec![]),
    )?;
  }
  /// Done
  Ok((shared_link_ah, owner_link_ah))
}



///
#[hdk_extern]
pub fn probe_shareds(_: ()) -> ExternResult<Vec<ActionHash>> {
  std::panic::set_hook(Box::new(zome_panic_hook));
  let root_path = Path::from(ROOT_ANCHOR_SHAREDS).typed(SharedOwnershipLinkType::SharedPath)?;
  let ph = root_path.path_entry_hash()?;
  let links = get_links(link_input(ph, SharedOwnershipLinkType::SharedEntry, None))?;
  /// Emit signal
  attest_links(links.clone())?;
  /// Done
  let shareds: Vec<ActionHash> = links
    .into_iter()
    .map(|link| link.target.into_action_hash().unwrap())
    .collect();
  Ok(shareds)
}


///
#[hdk_extern]
pub fn probe_owners(shared_ah: ActionHash) -> ExternResult<Vec<(AgentPubKey, ActionHash)>> {
  std::panic::set_hook(Box::new(zome_panic_hook));
  let links = get_links(link_input(shared_ah, SharedOwnershipLinkType::Owner, None))?;
  let pairs = links
    .into_iter()
    .map(|link| {
      let owner = link.target.into_agent_pub_key().unwrap();
      let tag_bytes = link.tag.clone().into_inner();
      let unsafe_bytes = UnsafeBytes::from(tag_bytes.clone());
      let ser_bytes = SerializedBytes::from(unsafe_bytes);
      let tag_owner: TagOwner = TagOwner::try_from(ser_bytes).unwrap();
      (owner, tag_owner.shared_link_ah)
    })
    .collect();
  /// Done
  Ok(pairs)
}

