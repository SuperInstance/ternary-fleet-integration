use std::sync::{Arc, Mutex};

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use ternary_fleet_integration::{
    aggregate_votes, generate_health_report, emit_fleet_pulse, FleetNode, FleetHealth,
};

/// Shared application state behind a mutex (simple, no db needed for a relay).
#[derive(Clone)]
struct AppState {
    nodes: Arc<Mutex<Vec<FleetNode>>>,
}

/// POST /api/pulse — ingest a batch of fleet node updates.
async fn post_pulse(
    State(state): State<AppState>,
    Json(nodes): Json<Vec<FleetNode>>,
) -> (StatusCode, Json<serde_json::Value>) {
    {
        let mut existing = state.nodes.lock().unwrap();
        // Replace known nodes by id, add new ones.
        for node in &nodes {
            if let Some(pos) = existing.iter().position(|n| n.id == node.id) {
                existing[pos] = node.clone();
            } else {
                existing.push(node.clone());
            }
        }
    }

    let pulse = emit_fleet_pulse(&nodes);
    let parsed: serde_json::Value = serde_json::from_str(&pulse).unwrap();
    (StatusCode::OK, Json(parsed))
}

/// GET /api/health — return fleet health report.
async fn get_health(State(state): State<AppState>) -> Json<FleetHealth> {
    let nodes = state.nodes.lock().unwrap();
    Json(generate_health_report(&nodes))
}

/// GET /api/votes — return aggregate ternary state.
async fn get_votes(State(state): State<AppState>) -> Json<serde_json::Value> {
    let nodes = state.nodes.lock().unwrap();
    let votes: Vec<i8> = nodes.iter().map(|n| n.ternary_vote).collect();
    let result = aggregate_votes(&votes);
    let response = serde_json::json!({
        "accept": result.accept,
        "neutral": result.neutral,
        "reject": result.reject,
        "total": result.total,
        "confidence": result.confidence,
        "nodes": nodes.len(),
    });
    Json(response)
}

#[tokio::main]
async fn main() {
    let state = AppState {
        nodes: Arc::new(Mutex::new(Vec::new())),
    };

    let app = Router::new()
        .route("/api/pulse", post(post_pulse))
        .route("/api/health", get(get_health))
        .route("/api/votes", get(get_votes))
        .with_state(state);

    let addr = "0.0.0.0:8790";
    println!("🚀 dash-relay listening on {addr}");
    println!("   POST /api/pulse  — ingest fleet node updates");
    println!("   GET  /api/health — fleet health report");
    println!("   GET  /api/votes  — aggregate ternary state");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
