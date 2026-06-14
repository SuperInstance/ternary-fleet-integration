use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Serialize;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use ternary_fleet_integration::{
    dash_emitter::emit_fleet_pulse,
    fleet_types::{FleetNode, TernaryVote},
    health_report::generate_health_report,
    ternary_aggregator::aggregate_votes,
};

#[derive(Clone)]
struct AppState {
    nodes: Arc<RwLock<Vec<FleetNode>>>,
    start_time: Instant,
}

#[derive(Serialize)]
struct PulseResponse {
    accepted: usize,
    message: String,
}

async fn handle_pulse(
    State(state): State<AppState>,
    Json(nodes): Json<Vec<FleetNode>>,
) -> (StatusCode, Json<PulseResponse>) {
    let count = nodes.len();
    let mut current = state.nodes.write().await;

    for node in nodes {
        if let Some(existing) = current.iter_mut().find(|n| n.id == node.id) {
            *existing = node;
        } else {
            current.push(node);
        }
    }

    (
        StatusCode::ACCEPTED,
        Json(PulseResponse {
            accepted: count,
            message: format!("{} nodes updated", count),
        }),
    )
}

async fn handle_health(State(state): State<AppState>) -> Json<serde_json::Value> {
    let nodes = state.nodes.read().await;
    let health = generate_health_report(&nodes, state.start_time);
    Json(serde_json::json!({
        "status": health.status,
        "node_count": health.node_count,
        "ternary_balance": health.ternary_balance,
        "uptime_secs": health.uptime_secs,
    }))
}

async fn handle_votes(State(state): State<AppState>) -> Json<serde_json::Value> {
    let nodes = state.nodes.read().await;
    let t_votes: Vec<TernaryVote> = nodes
        .iter()
        .map(|n| TernaryVote {
            node_id: n.id.clone(),
            proposal_id: "fleet-heartbeat".into(),
            vote: n.ternary_vote,
            weight: 1.0,
            timestamp: chrono::Utc::now(),
        })
        .collect();
    let result = aggregate_votes(&t_votes);

    Json(serde_json::json!({
        "total": result.total,
        "accept": result.accept,
        "neutral": result.neutral,
        "reject": result.reject,
        "confidence": result.confidence,
        "net_sentiment": result.net_sentiment,
        "pulse": emit_fleet_pulse(&nodes),
    }))
}

async fn handle_pulse_json(State(state): State<AppState>) -> Json<serde_json::Value> {
    let nodes = state.nodes.read().await;
    let pulse_str = emit_fleet_pulse(&nodes);
    if let Ok(pulse) = serde_json::from_str::<serde_json::Value>(&pulse_str) {
        Json(pulse)
    } else {
        Json(serde_json::json!({"error": "serialization failed"}))
    }
}

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8790".into())
        .parse()
        .expect("PORT must be a number");

    let state = AppState {
        nodes: Arc::new(RwLock::new(Vec::new())),
        start_time: Instant::now(),
    };

    let app = Router::new()
        .route("/api/pulse", post(handle_pulse))
        .route("/api/health", get(handle_health))
        .route("/api/votes", get(handle_votes))
        .route("/api/pulse", get(handle_pulse_json))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    println!("dash-relay listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
