//! Bridge ternary concepts to rate-limiting — map {-1, 0, +1} to request budgets and priority levels.

/// Map a ternary vote to a token budget multiplier.
/// +1 (accept) = full budget, 0 (neutral) = half budget, -1 (reject) = minimal budget.
pub fn ternary_token_budget(vote: i8) -> f64 {
    match vote.clamp(-1, 1) {
        1 => 1.0,
        0 => 0.5,
        -1 => 0.1,
        _ => 0.5,
    }
}

/// Map a ternary vote to a priority level (0-255).
/// +1 = highest priority, 0 = normal, -1 = lowest.
pub fn priority_from_ternary(vote: i8) -> u8 {
    match vote.clamp(-1, 1) {
        1 => 240,
        0 => 128,
        -1 => 16,
        _ => 128,
    }
}

/// Compute the aggregate rate limit from a set of ternary votes.
/// Returns a multiplier in [0.1, 1.0] based on fleet consensus.
pub fn fleet_rate_limit(votes: &[i8]) -> f64 {
    if votes.is_empty() {
        return 0.5;
    }
    let budget: f64 = votes.iter().map(|v| ternary_token_budget(*v)).sum();
    let rate = budget / votes.len() as f64;
    rate.clamp(0.1, 1.0)
}

/// Determine if a request should be throttled given the current ternary state and a threshold.
pub fn should_throttle(vote: i8, threshold: f64) -> bool {
    ternary_token_budget(vote) < threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_budget() {
        assert!((ternary_token_budget(1) - 1.0).abs() < 0.01);
        assert!((ternary_token_budget(0) - 0.5).abs() < 0.01);
        assert!((ternary_token_budget(-1) - 0.1).abs() < 0.01);
    }

    #[test]
    fn test_priority_mapping() {
        assert_eq!(priority_from_ternary(1), 240);
        assert_eq!(priority_from_ternary(0), 128);
        assert_eq!(priority_from_ternary(-1), 16);
    }

    #[test]
    fn test_fleet_rate_limit_mixed() {
        let votes = vec![1, 0, -1];
        let rate = fleet_rate_limit(&votes);
        // (1.0 + 0.5 + 0.1) / 3 ≈ 0.533
        assert!((rate - 0.533).abs() < 0.01);
    }

    #[test]
    fn test_fleet_rate_limit_empty() {
        let rate = fleet_rate_limit(&[]);
        assert!((rate - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_throttle() {
        assert!(!should_throttle(1, 0.5)); // 1.0 >= 0.5
        assert!(should_throttle(-1, 0.5)); // 0.1 < 0.5
    }
}
