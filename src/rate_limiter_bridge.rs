/// Compute a rate-limit token budget from a set of ternary votes.
///
/// Returns a value in [0.0, 1.0] where:
/// - 1.0 = full request budget (all accept)
/// - 0.5 = reduced budget (mixed / neutral)
/// - 0.0 = no budget allowed (all reject)
pub fn ternary_token_budget(votes: &[i8]) -> f64 {
    if votes.is_empty() {
        return 0.5;
    }

    let sum: f64 = votes.iter().map(|&v| v as f64).sum();
    let max_possible = votes.len() as f64;

    // Normalise from [-max_possible, +max_possible] to [0, 1]
    let budget = (sum / max_possible + 1.0) / 2.0;
    budget.clamp(0.0, 1.0)
}

/// Map a ternary vote to a priority level for the rate-limiter.
///
/// - +1 (accept) → priority 3 (highest)
/// -  0 (neutral) → priority 2
/// - -1 (reject)  → priority 1 (lowest)
pub fn priority_from_ternary(vote: i8) -> u8 {
    match vote {
        1 => 3,
        0 => 2,
        -1 => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_budget_all_accept() {
        let budget = ternary_token_budget(&[1, 1, 1]);
        assert!((budget - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_token_budget_all_reject() {
        let budget = ternary_token_budget(&[-1, -1, -1]);
        assert!((budget - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_token_budget_mixed() {
        let budget = ternary_token_budget(&[1, 0, -1]);
        // sum=0, max_possible=3, budget = (0/3+1)/2 = 0.5
        assert!((budget - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_priority_mapping() {
        assert_eq!(priority_from_ternary(1), 3);
        assert_eq!(priority_from_ternary(0), 2);
        assert_eq!(priority_from_ternary(-1), 1);
        assert_eq!(priority_from_ternary(42), 0);
    }
}
