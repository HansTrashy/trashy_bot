use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct State {
    pub en: String,
    pub de: String,
}

impl State {
    pub fn load() -> Self {
        match std::fs::read_to_string("rules_state.storage") {
            Ok(data) => {
                serde_json::from_str::<Self>(&data).expect("Could not deserialize rules state")
            }

            Err(_e) => Self {
                en: String::default(),
                de: String::default(),
            },
        }
    }
}
