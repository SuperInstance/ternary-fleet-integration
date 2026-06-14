use serde::Serialize;

use crate::fleet_types::FleetNode;

/// Health check response compatible with api-gateway's health endpoint.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FleetHealth {
    pub status: String,
    pub node_count: usize,
    pub ternary_balance: f64,
    pub uptime: chrono::DateTime<chrono::Utc>,
}

/// Generate a health report from the current set of fleet nodes.
///
/// `ternary_balance` is the mean of all ternary votes, giving a quick
/// snapshot of fleet consensus direction.
pub fn generate_health_report(nodes: &[FleetNode]) -> FleetHealth {
    let ternary_balance = if nodes.is_empty() {
        0.0
    } else {
        nodes
            .iter()
            .map(|n| n.ternary_vote as f64)
            .sum::<f64>()
            / nodes.len() as f64
    };

    FleetHealth {
        status: if ternary_balance >= 0.0 {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        },
        node_count: nodes.len(),
        ternary_balance,
        uptime: chrono::Utc::now(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fleet_types::FleetNode;

    #[test]
    fn test_healthy_fleet() {
        let nodes = vec![
            FleetNode {
                id: "node-1".into(),
                role: "validator".into(),
                capabilities: vec!["ternary".into()],
                ternary_vote: 1,
            },
            FleetNode {
                id: "node-2".into(),
                role: "validator".into(),
                capabilities: vec!["ternary".into()],
                ternary_vote: 0,
            },
            FleetNode {
                id: "node-3".into(),
                role: "observer".into(),
                capabilities: vec!["monitor".into()],
                ternary_vote: 1,
            },
        ];

        let report = generate_health_report(&nodes);
        assert_eq!(report.status, "healthy");
        assert_eq!(report.node_count, 3);
        assert!((report.ternary_balance - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_degraded_fleet() {
        let nodes = vec![
            FleetNode {
                id: "node-1".into(),
                role: "validator".into(),
                capabilities: vec!["ternary".into()],
                ternary_vote: -1,
            },
            FleetNode {
                id: "node-2".into(),
                role: "validator".into(),
                capabilities: vec!["ternary".into()],
                ternary_vote: -1,
            },
        ];

        let report = generate_health_report(&nodes);
        assert_eq!(report.status, "degraded");
        assert_eq!(report.node_count, 2);
        assert!((report.ternary_balance - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_empty_fleet() {
        let report = generate_health_report(&[]);
        assert_eq!(report.status, "healthy");
        assert_eq!(report.node_count, 0);
        assert_eq!(report.ternary_balance, 0.0);
    }
}
