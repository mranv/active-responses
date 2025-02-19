use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ThreatEvent {
    pub event_type: String,
    pub severity: i32,
    // Additional threat details can be added here
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseAction {
    pub action: String,
    // Additional action parameters can be added here
}
