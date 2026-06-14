//! Common types for fleet integration — the shared vocabulary between ternary math and forge infrastructure.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A node in the fleet: any agent, service, or instance that participates in ternary coordination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetNode {
    pub id: String,
    pub role: String,
    pub capabilities: Vec<String>,
    pub ternary_vote: i8, // -1 (reject), 0 (neutral), +1 (accept)
    pub last_seen: DateTime<Utc>,
    pub metrics: Vec<MetricSample>,
}

/// An event emitted by a fleet node, carrying a ternary merit signal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetEvent {
    pub event_type: String,
    pub source: String,
    pub payload: String,
    pub ternary_merit: i8,
    pub timestamp: DateTime<Utc>,
}

/// A single metric sample — the unit of telemetry in the fleet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSample {
    pub name: String,
    pub value: f64,
    pub tags: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

/// A ternary vote cast by a node on some proposal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TernaryVote {
    pub node_id: String,
    pub proposal_id: String,
    pub vote: i8, // -1, 0, +1
    pub weight: f64,
    pub timestamp: DateTime<Utc>,
}

impl FleetNode {
    pub fn new(id: &str, role: &str) -> Self {
        Self {
            id: id.to_string(),
            role: role.to_string(),
            capabilities: vec![],
            ternary_vote: 0,
            last_seen: Utc::now(),
            metrics: vec![],
        }
    }
}
