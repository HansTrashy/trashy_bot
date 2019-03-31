pub enum State {
        NotSet,
        Set {
            channel_id: u64,
            message_ids: Vec<u64>,
        },
    }

    impl State {
        pub fn set(channel_id: u64, message_ids: Vec<u64>) -> State {
            // also save those settings
            let data = serde_json::to_string(&(channel_id, &message_ids))
                .expect("Could not serialize rr state");
            std::fs::write("rr_state.storage", data).expect("coult not write rr state to file");

            State::Set {
                channel_id,
                message_ids,
            }
        }

        pub fn unset() -> State {
            State::NotSet
        }

        pub fn load_set() -> State {
            match std::fs::read_to_string("rr_state.storage") {
                Ok(data) => {
                    let (channel_id, message_ids): (u64, Vec<u64>) =
                        serde_json::from_str(&data).expect("could not deserialize rr state");
                    State::Set {
                        channel_id,
                        message_ids,
                    }
                }
                Err(_e) => State::NotSet,
            }
        }
    }