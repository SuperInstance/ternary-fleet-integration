//! Emit metrics and events in a format the fleet-dashboard can consume.

use crate::fleet_types::{FleetNode, MetricSample, TernaryVote};
use chrono::Utc;
use serde::Serialize;

/// A pulse payload — emitted periodically to broadcast fleet state.
#[derive(Debug, Serialize)]
pub struct FleetPulse {
    pub timestamp: String,
    pub nodes: Vec<FleetNodeSummary>,
    pub aggregate: AggregateSnapshot,
}

/// Summary view of a fleet node for dashboard consumption.
#[derive(Debug, Serialize)]
pub struct FleetNodeSummary {
    pub id: String,
    pub role: String,
    pub vote: i8,
    pub health: String,
}

/// An aggregated view of the fleet's ternary state.
#[derive(Debug, Serialize)]
pub struct AggregateSnapshot {
    pub total_nodes: usize,
    pub accept: usize,
    pub neutral: usize,
    pub reject: usize,
    pub consensus: f64,
}

/// Emit a fleet pulse as a JSON string, suitable for posting to fleet-dashboard backend.
pub fn emit_fleet_pulse(nodes: &[FleetNode]) -> String {
    let accept = nodes.iter().filter(|n| n.ternary_vote > 0).count();
    let neutral = nodes.iter().filter(|n| n.ternary_vote == 0).count();
    let reject = nodes.iter().filter(|n| n.ternary_vote < 0).count();
    let total = nodes.len();
    let consensus = if total > 0 {
        (accept as f64 - reject as f64) / total as f64
    } else {
        0.0
    };

    let pulse = FleetPulse {
        timestamp: Utc::now().to_rfc3339(),
        nodes: nodes
            .iter()
            .map(|n| FleetNodeSummary {
                id: n.id.clone(),
                role: n.role.clone(),
                vote: n.ternary_vote,
                health: if n.ternary_vote >= 0 { "green" } else { "yellow" }.to_string(),
            })
            .collect(),
        aggregate: AggregateSnapshot {
            total_nodes: total,
            accept,
            neutral,
            reject,
            consensus,
        },
    };

    serde_json::to_string_pretty(&pulse).unwrap_or_default()
}

/// Emit a ternary event as a JSON string for the event-bus.
pub fn emit_ternary_event(vote: &TernaryVote) -> String {
    serde_json::json!({
        "event_type": "ternary_vote",
        "source": vote.node_id,
        "proposal_id": vote.proposal_id,
        "vote": vote.vote,
        "weight": vote.weight,
        "timestamp": vote.timestamp.to_rfc3339(),
    })
    .to_string()
}

/// Emit a metric sample as a JSON line for the log-aggregator.
pub fn emit_metric(sample: &MetricSample) -> String {
    serde_json::json!({
        "metric": sample.name,
        "value": sample.value,
        "tags": sample.tags,
        "timestamp": sample.timestamp.to_rfc3339(),
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    // use chrono::Duration;

    #[test]
    fn test_emit_fleet_pulse_json() {
        let nodes = vec![
            FleetNode {
                id: "node-1".into(),
                role: "worker".into(),
                capabilities: vec!["compute".into()],
                ternary_vote: 1,
                metrics: vec![],
                last_seen: Utc::now(),
            },
            FleetNode {
                id: "node-2".into(),
                role: "validator".into(),
                capabilities: vec!["audit".into()],
                ternary_vote: -1,
                metrics: vec![],
                last_seen: Utc::now(),
            },
        ];

        let pulse = emit_fleet_pulse(&nodes);
        assert!(pulse.contains("node-1"));
        assert!(pulse.contains("node-2"));
        assert!(pulse.contains("consensus"));
    }

    #[test]
    fn test_emit_ternary_event() {
        let vote = TernaryVote {
            node_id: "oracle2".into(),
            proposal_id: "prop-42".into(),
            vote: 1,
            weight: 1.0,
            timestamp: Utc::now(),
        };
        let json = emit_ternary_event(&vote);
        assert!(json.contains("ternary_vote"));
        assert!(json.contains("oracle2"));
    }
}
