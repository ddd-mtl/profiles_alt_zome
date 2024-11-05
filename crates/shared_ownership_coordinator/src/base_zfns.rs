use hdk::prelude::*;
use zome_utils::*;
use zome_signals::*;
use shared_ownership_integrity::*;


///
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishOwnershipInput {
  pub agent: AgentPubKey,
  pub shared_ah: ActionHash,
  pub signature: Signature,
  pub as_author: bool,
}


/// Return (SharedLinkAh, OwnerLinkAh)
#[hdk_extern]
#[feature(zits_blocking)]
pub fn publish_ownership(input: PublishOwnershipInput) -> ExternResult<(ActionHash, ActionHash)> {
  std::panic::set_hook(Box::new(zome_panic_hook));
  /// TODO: check signature?
  /// Get owner proof
  let maybe_owner_link_ah = if input.as_author {
    None
  } else {
      let owners = get_owners(input.shared_ah.clone())?;
      let Some(pair) = owners.iter().filter(|&(owner, link_ah)| owner == &input.agent).next()
        else { return zome_error!("Agent is not an owner of shared entry"); };
      // for (owner, link_ah) in owners {
      //   if owner == input.agent {
      //     return link_ah;
      //   }
      // }
      // return zome_error!("Agent is not an owner of shared entry");
      Some(pair.clone().1)
  };
  /// Create tag
  let tag: TagShared = TagShared {
    signature: input.signature,
    maybe_owner_link_ah,
  };
  /// Create Shared link
  let shared_link_ah = create_link(input.agent.clone(), input.shared_ah.clone(), SharedOwnershipLinkType::Shared, obj2Tag(tag)?)?;
  /// Create OwnerLink
  let tag: TagOwner = TagOwner { shared_link_ah: shared_link_ah.clone() };
  let owner_link_ah = create_link(input.shared_ah.clone(), input.agent, SharedOwnershipLinkType::Owner, obj2Tag(tag)?)?;
  /// Create SharedPath Link
  if input.as_author {
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
pub fn pull_shareds(_: ()) -> ExternResult<Vec<ActionHash>> {
  std::panic::set_hook(Box::new(zome_panic_hook));
  let root_path = Path::from(ROOT_ANCHOR_SHAREDS).typed(SharedOwnershipLinkType::SharedPath)?;
  let ph = root_path.path_entry_hash()?;
  let links = get_links(link_input(ph, SharedOwnershipLinkType::SharedEntry, None))?;
  /// Emit signal
  emit_links_signal(links.clone())?;
  /// Done
  let shareds: Vec<ActionHash> = links
    .into_iter()
    .map(|link| link.target.into_action_hash().unwrap())
    .collect();
  Ok(shareds)
}


///
#[hdk_extern]
pub fn get_owners(shared_ah: ActionHash) -> ExternResult<Vec<(AgentPubKey, ActionHash)>> {
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


///
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfferOwnershipInput {
  pub agent: AgentPubKey,
  pub shared_ah: ActionHash,
  //pub signature: Sign,
  //pub as_author: bool,
}


///
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppTip {
  #[serde(rename = "type")]
  type_type: String,
  shared_ah: ActionHash,
  maybe_sign: Option<Signature>,
}


///
#[hdk_extern]
pub fn offer_ownership(input: OfferOwnershipInput) -> ExternResult<()> {
  let app_tip = AppTip {
    type_type: "offer".to_string(),
    shared_ah: input.shared_ah,
    maybe_sign: None,
  };
  let data = encode(&app_tip).unwrap();
  let tip: TipProtocol = TipProtocol::App(UnsafeBytes::from(data).into());
  return cast_tip(CastTipInput {tip, peers: vec![input.agent]});
}


///
#[hdk_extern]
pub fn request_ownership(input: PublishOwnershipInput) -> ExternResult<()> {
  let app_tip = AppTip {
    type_type: "request".to_string(),
    shared_ah: input.shared_ah,
    maybe_sign: Some(input.signature),
  };
  let data = encode(&app_tip).unwrap();
  let tip: TipProtocol = TipProtocol::App(UnsafeBytes::from(data).into());
  return cast_tip(CastTipInput {tip, peers: vec![input.agent]});
}