/// Result of aggregating ternary votes from fleet nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct AggregateResult {
    pub accept: usize,
    pub neutral: usize,
    pub reject: usize,
    pub total: usize,
    pub confidence: f64,
}

/// Aggregate raw ternary votes into a summary result with confidence.
///
/// Confidence is computed as the ratio of the winning vote count to total votes,
/// on a scale where 0.5 is the minimum meaningful signal (pure randomness would
/// give ~0.33 for three outcomes).
pub fn aggregate_votes(votes: &[i8]) -> AggregateResult {
    let accept = votes.iter().filter(|&&v| v == 1).count();
    let neutral = votes.iter().filter(|&&v| v == 0).count();
    let reject = votes.iter().filter(|&&v| v == -1).count();
    let total = votes.len();

    let confidence = if total == 0 {
        0.0
    } else {
        let max_count = accept.max(neutral).max(reject) as f64;
        (max_count / total as f64 - 1.0 / 3.0) / (1.0 - 1.0 / 3.0)
    };

    AggregateResult {
        accept,
        neutral,
        reject,
        total,
        confidence,
    }
}

/// Compute a weighted soft consensus value from (vote, weight) pairs.
///
/// Returns a value in [-1.0, 1.0] where:
/// - 1.0  = unanimous accept
/// - 0.0  = perfectly balanced / neutral
/// - -1.0 = unanimous reject
pub fn weighted_consensus(votes: &[(i8, f64)]) -> f64 {
    if votes.is_empty() {
        return 0.0;
    }

    let total_weight: f64 = votes.iter().map(|(_, w)| w).sum();
    if total_weight <= 0.0 {
        return 0.0;
    }

    let weighted_sum: f64 = votes.iter().map(|(v, w)| (*v as f64) * w).sum();
    (weighted_sum / total_weight).clamp(-1.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregation_exact_match() {
        let votes = vec![1, 1, 1, -1, 0, 1, 1, 1];
        let result = aggregate_votes(&votes);
        assert_eq!(result.accept, 6);
        assert_eq!(result.neutral, 1);
        assert_eq!(result.reject, 1);
        assert_eq!(result.total, 8);
        // Confidence: (6/8 - 1/3) / (1 - 1/3) = (0.75 - 0.333) / 0.667 ≈ 0.625
        assert!((result.confidence - 0.625).abs() < 0.01);
        assert!(result.confidence <= 1.0);
    }

    #[test]
    fn test_aggregation_weighted() {
        // Strong accept votes with high weights
        let votes = vec![(1, 10.0), (0, 1.0), (-1, 0.5), (1, 8.0)];
        let consensus = weighted_consensus(&votes);
        // Weighted sum: 10 + 0 - 0.5 + 8 = 17.5 / 19.5 ≈ 0.897
        let expected = (10.0 + 0.0 - 0.5 + 8.0) / (10.0 + 1.0 + 0.5 + 8.0);
        assert!((consensus - expected).abs() < 1e-10);
        assert!(consensus > 0.8);
    }

    #[test]
    fn test_empty_votes() {
        let result = aggregate_votes(&[]);
        assert_eq!(result.total, 0);
        assert_eq!(result.confidence, 0.0);

        let consensus = weighted_consensus(&[]);
        assert_eq!(consensus, 0.0);
    }

    #[test]
    fn test_perfect_unanimity() {
        let votes = vec![1, 1, 1, 1];
        let result = aggregate_votes(&votes);
        assert_eq!(result.confidence, 1.0);
    }

    #[test]
    fn test_perfect_deadlock() {
        let votes = vec![1, 0, -1];
        let result = aggregate_votes(&votes);
        assert_eq!(result.accept, 1);
        assert_eq!(result.neutral, 1);
        assert_eq!(result.reject, 1);
        // Confidence should be 0.0 — one vote each, no signal
        assert!(result.confidence.abs() < 1e-10);
    }
}
