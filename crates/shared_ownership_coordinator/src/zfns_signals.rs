use hdk::prelude::*;
use zome_signals::*;


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
  pub type_type: String,
  pub shared_ah: ActionHash,
  pub maybe_sign: Option<Signature>,
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
  let tip: TipProtocol = TipProtocol::AppCustom(UnsafeBytes::from(data).into());
  return cast_tip(CastTipInput {tip, peers: vec![input.agent]});
}



///
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestOwnershipInput {
  pub agent: AgentPubKey,
  pub shared_ah: ActionHash,
  pub signature: Signature,
}


///
#[hdk_extern]
pub fn request_ownership(input: RequestOwnershipInput) -> ExternResult<()> {
  let app_tip = AppTip {
    type_type: "request_ownership".to_string(),
    shared_ah: input.shared_ah,
    maybe_sign: Some(input.signature),
  };
  let data = encode(&app_tip).unwrap();
  let tip: TipProtocol = TipProtocol::AppCustom(UnsafeBytes::from(data).into());
  return cast_tip(CastTipInput {tip, peers: vec![input.agent]});
}
