use serenity::model::channel::PermissionOverwrite;
use std::collections::HashMap;

pub struct State {
    old_permissions: HashMap<u64, Vec<PermissionOverwrite>>,
}

impl State {
    pub fn insert(&mut self, channel_id: u64, ows: Vec<PermissionOverwrite>) {
        // also save those settings
        // let data = serde_json::to_string(&(channel_id, &message_ids))
        //     .expect("Could not serialize rr state");
        // std::fs::write("rr_state.storage", data).expect("coult not write rr state to file");
        self.old_permissions.insert(channel_id, ows);
    }

    pub fn is_active(&mut self, channel_id: u64) -> bool {
        self.old_permissions.contains_key(&channel_id)
    }

    pub fn remove(&mut self, channel_id: u64) {
        self.old_permissions.remove(&channel_id);
    }

    // pub fn load_set() -> State {
    //     match std::fs::read_to_string("rr_state.storage") {
    //         Ok(data) => {
    //             let (channel_id, message_ids): (u64, Vec<u64>) =
    //                 serde_json::from_str(&data).expect("could not deserialize rr state");
    //             State::Set {
    //                 channel_id,
    //                 message_ids,
    //             }
    //         }
    //         Err(_e) => State::NotSet,
    //     }
    // }
}
