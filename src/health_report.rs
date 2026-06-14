//! Health reporting — produce responses compatible with the api-gateway health endpoint.

use crate::fleet_types::FleetNode;
use serde::Serialize;
use std::time::Instant;

/// A fleet-wide health report — the response the api-gateway returns for /health.
#[derive(Debug, Serialize)]
pub struct FleetHealth {
    pub status: String,
    pub node_count: usize,
    pub ternary_balance: f64,   // net sentiment across all nodes
    pub uptime_secs: u64,
}

/// Generate a health report from the current fleet nodes.
pub fn generate_health_report(nodes: &[FleetNode], start_time: Instant) -> FleetHealth {
    let total = nodes.len();
    let accept = nodes.iter().filter(|n| n.ternary_vote > 0).count();
    let reject = nodes.iter().filter(|n| n.ternary_vote < 0).count();

    let ternary_balance = if total > 0 {
        (accept as f64 - reject as f64) / total as f64
    } else {
        0.0
    };

    // Health status: green if positive balance, yellow if neutral, red if negative
    let status = if ternary_balance > 0.2 {
        "green"
    } else if ternary_balance >= -0.2 {
        "yellow"
    } else {
        "red"
    };

    FleetHealth {
        status: status.to_string(),
        node_count: total,
        ternary_balance,
        uptime_secs: start_time.elapsed().as_secs(),
    }
}

/// Quick check if a single node is healthy based on its ternary vote.
pub fn node_healthy(node: &FleetNode) -> bool {
    node.ternary_vote >= 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::time::Instant;

    #[test]
    fn test_healthy_fleet() {
        let nodes = vec![
            FleetNode {
                id: "alice".into(),
                role: "worker".into(),
                capabilities: vec![],
                ternary_vote: 1,
                metrics: vec![],
                last_seen: Utc::now(),
            },
            FleetNode {
                id: "bob".into(),
                role: "worker".into(),
                capabilities: vec![],
                ternary_vote: 1,
                metrics: vec![],
                last_seen: Utc::now(),
            },
        ];
        let report = generate_health_report(&nodes, Instant::now());
        assert_eq!(report.status, "green");
        assert_eq!(report.node_count, 2);
        assert!((report.ternary_balance - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_unhealthy_fleet() {
        let nodes = vec![
            FleetNode {
                id: "mallory".into(),
                role: "worker".into(),
                capabilities: vec![],
                ternary_vote: -1,
                metrics: vec![],
                last_seen: Utc::now(),
            },
        ];
        let report = generate_health_report(&nodes, Instant::now());
        assert_eq!(report.status, "red");
        assert!((report.ternary_balance - (-1.0)).abs() < 0.01);
    }

    #[test]
    fn test_node_healthy() {
        let healthy = FleetNode {
            id: "h".into(),
            role: "worker".into(),
            capabilities: vec![],
            ternary_vote: 1,
            metrics: vec![],
            last_seen: Utc::now(),
        };
        let unhealthy = FleetNode {
            id: "u".into(),
            role: "worker".into(),
            capabilities: vec![],
            ternary_vote: -1,
            metrics: vec![],
            last_seen: Utc::now(),
        };
        assert!(node_healthy(&healthy));
        assert!(!node_healthy(&unhealthy));
    }
}
