mod callbacks;
mod zfns_base;
mod zfns_signals;
mod zfns_key;

use hdk::prelude::*;

#[hdk_extern]
fn get_zome_info(_:()) -> ExternResult<ZomeInfo> {
  return zome_info();
}


#[hdk_extern]
fn get_dna_info(_:()) -> ExternResult<DnaInfo> {
  return dna_info();
}


#[hdk_extern]
fn get_record_author(dh: AnyDhtHash) -> ExternResult<AgentPubKey> {
  return zome_utils::get_author(dh);
}