use serde::{Deserialize, Serialize};

/// A node within the Forgemaster fleet.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FleetNode {
    pub id: String,
    pub role: String,
    pub capabilities: Vec<String>,
    pub ternary_vote: i8,
}

/// An event flowing through the fleet event-bus.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FleetEvent {
    pub event_type: String,
    pub source: String,
    pub payload: serde_json::Value,
    pub ternary_merit: i8,
}

/// A single metric sample for the fleet dashboard.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetricSample {
    pub name: String,
    pub value: f64,
    pub tags: Vec<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
