use hdk::prelude::*;
use hc_zome_profiles_integrity::*;


///
pub fn prefix_path(nickname: String) -> ExternResult<TypedPath> {
   // convert to lowercase for path for ease of search
   let lower_nickname = nickname.to_lowercase();
   let prefix: String = lower_nickname.chars().take(3).collect();

   Path::from(format!("all_profiles.{}", prefix)).typed(LinkTypes::PrefixPath)
}




///
pub fn get_entry_for_action(action_hash: &ActionHash) -> ExternResult<Option<EntryTypes>> {
   let record = match get_details(action_hash.clone(), GetOptions::default())? {
      Some(Details::Record(record_details)) => record_details.record,
      _ => {
         return Ok(None);
      }
   };
   let entry = match record.entry().as_option() {
      Some(entry) => entry,
      None => {
         return Ok(None);
      }
   };
   let (zome_index, entry_index) = match record.action().entry_type() {
      Some(EntryType::App(AppEntryDef {
                             zome_index,
                             entry_index,
                             ..
                          })) => (zome_index, entry_index),
      _ => {
         return Ok(None);
      }
   };
   Ok(EntryTypes::deserialize_from_type(
      zome_index.clone(),
      entry_index.clone(),
      entry,
   )?)
}
