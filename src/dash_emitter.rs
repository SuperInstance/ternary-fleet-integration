use crate::fleet_types::FleetNode;

/// Emit a fleet pulse JSON blob consumable by the fleet-dashboard backend.
pub fn emit_fleet_pulse(nodes: &[FleetNode]) -> String {
    #[derive(serde::Serialize)]
    struct FleetPulse<'a> {
        pulse_type: &'a str,
        nodes: &'a [FleetNode],
        node_count: usize,
        ternary_mean: f64,
    }

    let ternary_mean = if nodes.is_empty() {
        0.0
    } else {
        nodes.iter().map(|n| n.ternary_vote as f64).sum::<f64>() / nodes.len() as f64
    };

    let pulse = FleetPulse {
        pulse_type: "fleet_pulse",
        nodes,
        node_count: nodes.len(),
        ternary_mean,
    };

    serde_json::to_string_pretty(&pulse).unwrap()
}

/// Emit a ternary event as a JSON string for the event bus.
pub fn emit_ternary_event(vote: i8) -> String {
    #[derive(serde::Serialize)]
    struct TernaryEvent {
        event_type: String,
        vote: i8,
        label: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    }

    let label = match vote {
        -1 => "reject",
        0 => "neutral",
        1 => "accept",
        _ => "invalid",
    }
    .to_string();

    let event = TernaryEvent {
        event_type: "ternary_vote".to_string(),
        vote,
        label,
        timestamp: chrono::Utc::now(),
    };

    serde_json::to_string_pretty(&event).unwrap()
}
