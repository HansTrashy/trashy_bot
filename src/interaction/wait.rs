use chrono::prelude::*;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Action {
    DeleteFav,
}

#[derive(Debug)]
pub struct WaitEvent {
    action: Action,
    action_id: i64,
    timestamp: DateTime<Utc>,
}

impl WaitEvent {
    pub fn new(action: Action, action_id: i64, timestamp: DateTime<Utc>) -> Self {
        Self {
            action,
            action_id,
            timestamp,
        }
    }
}

pub struct Wait(HashMap<u64, Vec<WaitEvent>>);

impl Wait {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn wait(&mut self, user_id: u64, event: WaitEvent) {
        let wait_events = self.0.entry(user_id).or_insert_with(Vec::new);
        // println!("added wait for: {}, event: {:?}", &user_id, &event);
        wait_events.push(event);
    }

    pub fn waiting(&mut self, user_id: u64, action: Action) -> Option<i64> {
        let wait_events = self.0.entry(user_id).or_insert_with(Vec::new);

        // println!("Check if waiting for something of user: {}", &user_id);

        let mut was_waiting = None;
        wait_events.retain(|w| {
            // dbg!(&w);
            if w.action == action {
                // println!("Found something!");
                was_waiting = Some(w.action_id);
                false
            } else {
                true
            }
        });
        was_waiting
    }
}
