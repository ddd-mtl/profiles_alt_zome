use hdk::prelude::*;
use zome_utils::*;
use zome_signals::*;
use shared_ownership_integrity::*;
use crate::zfns_base::probe_owners;
use crate::zfns_signals::AppTip;

///
#[hdk_extern]
#[feature(zits_blocking)]
pub fn create_shared_key(_: ()) -> ExternResult<ActionHash> {
  std::panic::set_hook(Box::new(zome_panic_hook));
  let key_ref = x_salsa20_poly1305_shared_secret_create_random(None)?;
  let ah = create_entry(SharedOwnershipEntry::SharedKey(SharedKey { key_ref }))?;
  Ok(ah)
}


///
pub fn encrypt_with_shared_key<T>(data: T, key_ah: ActionHash) -> ExternResult<XSalsa20Poly1305EncryptedData>
where
  T: serde::Serialize + Clone + Sized + std::fmt::Debug
{
  /// Serialize
  let data: XSalsa20Poly1305Data = bincode::serialize(&data).unwrap().into();
  /// Get Key
  let (_eh, key) = get_typed_from_ah::<SharedKey>(key_ah.clone())?;
  /// Encrypt
  let enc_data = x_salsa20_poly1305_encrypt(key.key_ref, data)?;
  Ok(enc_data)
}

///
pub fn decrypt_with_shared_key<T>(enc_data: XSalsa20Poly1305EncryptedData, key_ah: ActionHash) -> ExternResult<T>
where
  T: for<'a> serde::Deserialize<'a> + Clone + Sized + std::fmt::Debug
{
  /// Get Key
  let (_eh, key) = get_typed_from_ah::<SharedKey>(key_ah.clone())?;
  /// Encrypt
  let Some(ser_data) = x_salsa20_poly1305_decrypt(key.key_ref, enc_data)?
    else { return zome_error!("Failed to decrypt data")};
  /// Deserialize
  let data: T = bincode::deserialize(ser_data.as_ref()).unwrap();
  /// Done
  Ok(data)
}


///
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendKeyInput {
  pub recipient: AgentPubKey,
  pub key_ah: ActionHash,
}


#[hdk_extern]
#[feature(zits_blocking)]
pub fn send_shared_key(input: SendKeyInput) -> ExternResult<()> {
  let (_eh, key) = get_typed_from_ah::<SharedKey>(input.key_ah.clone())?;
  let send = RecvKeyInput {key_ah: input.key_ah, key};
  let resp = call_remote(input.recipient, DEFAULT_COORDINATOR_ZOME_NAME, FunctionName("recv_shared_key".into()), None, send)?;
  decode_response(resp)?;
  Ok(())
}


///
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecvKeyInput {
  pub key_ah: ActionHash,
  pub key: SharedKey,
}


#[hdk_extern]
#[feature(zits_blocking)]
pub fn recv_shared_key(input: RecvKeyInput) -> ExternResult<()> {
  //TODO: check if we have published ownership of this key
  let _ah = create_entry(SharedOwnershipEntry::SharedKey(SharedKey { key_ref: input.key.key_ref }))?;
  /// TODO: maybe need to add it to lair?
  // let key_ref = x_salsa20_poly1305_shared_secret_create_random(Some(input.key.key_ref))?;
  /// DONE
  Ok(())
}


///
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestKeyInput {
  pub maybe_agent: Option<AgentPubKey>,
  pub shared_ah: ActionHash,
}


///
#[hdk_extern]
pub fn request_shared_key(input: RequestKeyInput) -> ExternResult<()> {
  let app_tip = AppTip {
    type_type: "request_key".to_string(),
    shared_ah: input.shared_ah.clone(),
    maybe_sign: None,
  };
  let data = encode(&app_tip).unwrap();
  let tip: TipProtocol = TipProtocol::App(UnsafeBytes::from(data).into());
  ///
  let agent = if let Some(agent) = input.maybe_agent { agent } else {
    let owners = probe_owners(input.shared_ah)?;
    if owners.is_empty() {
      return zome_error!("No owners found for shared key");
    }
    // TODO: pick random owner
    owners[0].0.clone()
  };
  ///
  return cast_tip(CastTipInput {tip, peers: vec![agent]});
}


///
#[hdk_extern]
pub fn query_shared_keys(_: ()) -> ExternResult<Vec<ActionHash>> {
  std::panic::set_hook(Box::new(zome_panic_hook));
  /// Query type
  let query_args = ChainQueryFilter::default()
    .include_entries(false)
    .action_type(ActionType::Create)
    //.action_type(ActionType::Update)
    .entry_type(SharedOwnershipEntryTypes::SharedKey.try_into().unwrap());
  let records = query(query_args)?;
  /// Get entries for all results
  let mut key_ahs: Vec<ActionHash> = Vec::new();
  for record in records {
    key_ahs.push(record.action_hashed().clone().into_hash())
  }
  /// Done
  Ok(key_ahs)
}
