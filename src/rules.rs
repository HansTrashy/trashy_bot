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
                serde_json::from_str::<State>(&data).expect("could not deserialize rules state")
            }

            Err(_e) => Self {
                en: String::default(),
                de: String::default(),
            },
        }
    }

    pub fn set_en(&mut self, en: &str) {
        let data = serde_json::to_string(&self).expect("Could not serialize rules state");
        std::fs::write("rules_state.storage", data).expect("coult not write rules state to file");

        self.en = en.to_string();
    }

    pub fn set_de(&mut self, de: &str) {
        let data = serde_json::to_string(&self).expect("Could not serialize rules state");
        std::fs::write("rules_state.storage", data).expect("coult not write rules state to file");

        self.de = de.to_string();
    }
}
