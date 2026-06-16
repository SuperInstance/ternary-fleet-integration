# CROSS-POLLINATION.md — ternary-fleet-integration

> **Conservation Law Connection:** Verifies γ quality across fleet components

## Role in the Conservation Law

`ternary-fleet-integration` is the test harness that verifies fleet sub-crates
produce correct results when composed. In conservation law terms, it tests **γ integrity**:
when multiple γ-contributing components are chained, does the output quality degrade
beyond what η predicts?

If integration test quality drops below (1 − δ(n)) × individual_quality, the conservation
law is violated at the integration layer, indicating architectural problems.

## delta-clt Verification Results

The delta-clt independent fleet simulation shows that composing n independent
components should yield quality ≈ 1 − δ(n) per component. The integration tests
should confirm:

- 2 components composed: expected quality ≥ 1 − δ(2) = 1 − 0.189 = 81.1%
- 5 components composed: expected quality ≥ 1 − δ(5) = 1 − 0.333 = 66.7%
- 10 components composed: expected quality ≥ 1 − δ(10) = 1 − 0.217 = 78.3%

If integration quality falls below these thresholds, the components have hidden
correlations (like the delta-clt correlated fleet simulation).

## Cross-Repo Connections

### → ternary-fleet
Directly tests the sub-crates of ternary-fleet. Every integration test exercises
the γ path of the conservation law.

**Shared:** Same crate ecosystem. Integration tests import fleet sub-crates directly.
**Different:** `fleet` implements; `integration` verifies.

### → conservation-action
`conservation-action` provides the GitHub Action that enforces γ + η ≤ C in CI.
`ternary-fleet-integration` should be the test phase that `conservation-action`
gates against. Together they form the conservation law enforcement pipeline.

**Shared:** Both are CI/CD infrastructure for conservation law compliance.
**Different:** `integration` tests functional correctness (γ); `conservation-action`
enforces the mathematical bound (γ + η ≤ C).

### → delta-clt
`delta-clt` provides the theoretical baselines that integration tests should be
compared against. If integration test results diverge from delta-clt predictions,
the fleet has architectural debt.

**Shared:** Both verify the conservation law holds.
**Different:** `delta-clt` is Monte Carlo simulation; `integration` is real code paths.

## Fleet Position

```
┌──────────────────────────────────────────────────┐
│  ternary-fleet-integration — THE γ VERIFIER       │
│                                                   │
│  ternary-fleet ──► INTEGRATION TESTS ──► PASS/FAIL│
│                          │                        │
│                          ▼                        │
│  Compare against delta-clt predictions:           │
│    If quality ≥ (1 − δ(n)): ✅ law holds          │
│    If quality < (1 − δ(n)): ⚠️ hidden correlation │
│                                                   │
│  Gated by: conservation-action (CI enforcement)   │
└──────────────────────────────────────────────────┘
```

