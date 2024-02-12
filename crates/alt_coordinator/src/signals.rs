use hdk::prelude::*;
use profiles_integrity::*;
use crate::utils::*;


#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Signal {
   LinkCreated {
      action: SignedActionHashed,
      link_type: LinkTypes,
   },
   LinkDeleted {
      action: SignedActionHashed,
      create_link_action: SignedActionHashed,
      link_type: LinkTypes,
   },
   EntryCreated {
      action: SignedActionHashed,
      app_entry: EntryTypes,
   },
   EntryUpdated {
      action: SignedActionHashed,
      app_entry: EntryTypes,
      original_app_entry: EntryTypes,
   },
   EntryDeleted {
      action: SignedActionHashed,
      original_app_entry: EntryTypes,
   },
}
#[hdk_extern(infallible)]
pub fn post_commit(committed_actions: Vec<SignedActionHashed>) {
   for action in committed_actions {
      if let Err(err) = signal_action(action) {
         error!("Error signaling new action: {:?}", err);
      }
   }
}
fn signal_action(action: SignedActionHashed) -> ExternResult<()> {
   match action.hashed.content.clone() {
      Action::CreateLink(create_link) => {
         if let Ok(Some(link_type)) =
            LinkTypes::from_type(create_link.zome_index, create_link.link_type)
         {
            emit_signal(Signal::LinkCreated { action, link_type })?;
         }
         Ok(())
      }
      Action::DeleteLink(delete_link) => {
         let record = get(delete_link.link_add_address.clone(), GetOptions::default())?.ok_or(
            wasm_error!(WasmErrorInner::Guest(
                    "Failed to fetch CreateLink action".to_string()
                )),
         )?;
         match record.action() {
            Action::CreateLink(create_link) => {
               if let Ok(Some(link_type)) =
                  LinkTypes::from_type(create_link.zome_index, create_link.link_type)
               {
                  emit_signal(Signal::LinkDeleted {
                     action,
                     link_type,
                     create_link_action: record.signed_action.clone(),
                  })?;
               }
               Ok(())
            }
            _ => {
               return Err(wasm_error!(WasmErrorInner::Guest(
                        "Create Link should exist".to_string()
                    )));
            }
         }
      }
      Action::Create(_create) => {
         if let Ok(Some(app_entry)) = get_entry_for_action(&action.hashed.hash) {
            emit_signal(Signal::EntryCreated { action, app_entry })?;
         }
         Ok(())
      }
      Action::Update(update) => {
         if let Ok(Some(app_entry)) = get_entry_for_action(&action.hashed.hash) {
            if let Ok(Some(original_app_entry)) =
               get_entry_for_action(&update.original_action_address)
            {
               emit_signal(Signal::EntryUpdated {
                  action,
                  app_entry,
                  original_app_entry,
               })?;
            }
         }
         Ok(())
      }
      Action::Delete(delete) => {
         if let Ok(Some(original_app_entry)) = get_entry_for_action(&delete.deletes_address) {
            emit_signal(Signal::EntryDeleted {
               action,
               original_app_entry,
            })?;
         }
         Ok(())
      }
      _ => Ok(()),
   }
}
