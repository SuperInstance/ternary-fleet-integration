//! Aggregate ternary votes from fleet nodes — settle on consensus.

use crate::fleet_types::{FleetNode, TernaryVote};

/// The result of aggregating ternary votes across a fleet.
#[derive(Debug, Clone)]
pub struct AggregateResult {
    pub total: usize,
    pub accept: usize,
    pub neutral: usize,
    pub reject: usize,
    pub confidence: f64,          // how decisive the vote is (0.0 = split, 1.0 = unanimous)
    pub net_sentiment: f64,       // weighted net: (accept - reject) / total
}

/// Aggregate raw {-1, 0, +1} votes into a result.
pub fn aggregate_votes(votes: &[TernaryVote]) -> AggregateResult {
    let total = votes.len();
    let accept = votes.iter().filter(|v| v.vote > 0).count();
    let neutral = votes.iter().filter(|v| v.vote == 0).count();
    let reject = votes.iter().filter(|v| v.vote < 0).count();

    // Confidence: 1.0 if all same, 0.0 if perfectly split
    let max_group = accept.max(neutral).max(reject) as f64;
    let confidence = if total > 0 { max_group / total as f64 } else { 0.0 };

    // Net sentiment in [-1, 1]
    let net_sentiment = if total > 0 {
        (accept as f64 - reject as f64) / total as f64
    } else {
        0.0
    };

    AggregateResult {
        total,
        accept,
        neutral,
        reject,
        confidence,
        net_sentiment,
    }
}

/// Weighted consensus — each vote has a weight, returns a soft consensus value in [-1, 1].
/// Useful when some nodes have more authority or more reliable signal.
pub fn weighted_consensus(votes: &[(i8, f64)]) -> f64 {
    let total_weight: f64 = votes.iter().map(|(_, w)| w.abs()).sum();
    if total_weight == 0.0 {
        return 0.0;
    }
    let weighted_sum: f64 = votes.iter().map(|(v, w)| (*v as f64) * w).sum();
    (weighted_sum / total_weight).clamp(-1.0, 1.0)
}

/// Extract vote-weight pairs from a slice of FleetNode using their ternary_vote.
pub fn votes_from_nodes(nodes: &[FleetNode]) -> Vec<(i8, f64)> {
    nodes
        .iter()
        .map(|n| (n.ternary_vote, 1.0))
        .collect()
}

/// Determine if a proposal reached consensus.
/// threshold: minimum fraction of votes that must agree (default 0.5)
pub fn has_consensus(result: &AggregateResult, threshold: f64) -> bool {
    let agree = result.accept.max(result.neutral).max(result.reject) as f64;
    result.total > 0 && agree / result.total as f64 >= threshold
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_vote(vote: i8) -> TernaryVote {
        TernaryVote {
            node_id: "test".into(),
            proposal_id: "test".into(),
            vote,
            weight: 1.0,
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_all_accept() {
        let votes: Vec<TernaryVote> = vec![make_vote(1), make_vote(1), make_vote(1)];
        let r = aggregate_votes(&votes);
        assert_eq!(r.accept, 3);
        assert_eq!(r.reject, 0);
        assert!((r.confidence - 1.0).abs() < 0.01);
        assert!((r.net_sentiment - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_exact_split() {
        let votes: Vec<TernaryVote> = vec![make_vote(1), make_vote(-1)];
        let r = aggregate_votes(&votes);
        assert_eq!(r.accept, 1);
        assert_eq!(r.reject, 1);
        assert!((r.confidence - 0.5).abs() < 0.01);
        assert!((r.net_sentiment - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_weighted_consensus_mixed() {
        let votes = vec![(1, 2.0), (1, 1.0), (-1, 1.0)];
        let result = weighted_consensus(&votes);
        // weighted sum = 1*2 + 1*1 + (-1)*1 = 2, total weight = 4, consensus = 0.5
        assert!((result - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_has_consensus() {
        let votes: Vec<TernaryVote> = vec![make_vote(1), make_vote(1), make_vote(-1)];
        let r = aggregate_votes(&votes);
        assert!(has_consensus(&r, 0.5));  // 2/3 > 0.5
        assert!(!has_consensus(&r, 0.75)); // 2/3 < 0.75
    }

    #[test]
    fn test_empty() {
        let votes = vec![];
        let r = aggregate_votes(&votes);
        assert_eq!(r.total, 0);
        assert!((r.confidence - 0.0).abs() < 0.01);
    }
}
