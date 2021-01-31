use tracing::warn;

/// Setting the `MessageId's` for the reaction messages

#[warn(dead_code)]
pub enum State {
    NotSet,
    Set(Vec<u64>), // message ids
}

impl State {
    pub fn get(&self) -> Option<Vec<u64>> {
        match self {
            Self::NotSet => None,
            Self::Set(ids) => Some(ids.clone()),
        }
    }

    pub fn set(message_ids: Vec<u64>) -> Self {
        // also save those settings
        let data = serde_json::to_string(&message_ids).expect("Could not serialize rr state");
        std::fs::write("rr_state.storage", data).expect("Could not write rr state to file");

        Self::Set(message_ids)
    }

    pub fn load_set() -> Self {
        match std::fs::read_to_string("rr_state.storage") {
            Ok(data) => {
                let message_ids: Vec<u64> =
                    serde_json::from_str(&data).expect("Could not deserialize rr state");
                Self::Set(message_ids)
            }
            Err(e) => {
                warn!(?e, "failed to read rr_state.storage");
                Self::NotSet
            }
        }
    }
}
