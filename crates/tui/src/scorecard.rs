//! Token / cache / cost scorecard (#3388).
//!
//! A release-gate view of an agent run's token economics: per-turn input /
//! output / cache-read tokens and cost, aggregate totals + cache-hit ratio, and
//! regression detection against a committed baseline. This is the measurement
//! layer the "token, cache, and context discipline" EPIC asks for — it makes a
//! cost/token regression visible instead of silently shipping.
//!
//! The core here is pure and offline: it turns already-recorded per-turn
//! [`Usage`] (captured on every turn, persisted in `TurnRecord`) into a
//! scorecard, reusing the existing pricing layer rather than reinventing cost
//! math. The `scorecard` subcommand is a thin I/O wrapper over this module.

use serde::{Deserialize, Serialize};

use crate::models::Usage;
use crate::pricing::{calculate_turn_cost_estimate_from_usage, token_usage_for_pricing};

/// One turn's normalized token economics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TurnScore {
    pub turn_id: String,
    pub model: String,
    /// Non-cached (billable) input tokens.
    pub input_tokens: u64,
    /// Output tokens, including reasoning output.
    pub output_tokens: u64,
    /// Cache-read (cache-hit) input tokens.
    pub cache_read_tokens: u64,
    pub cost_usd: f64,
    pub cost_cny: f64,
    /// True when no pricing row exists for `model`: cost is reported as 0 but is
    /// not meaningful, so the summary can flag it rather than imply "$0.00".
    pub cost_unpriced: bool,
}

/// Aggregate metrics for a run. Serializes/deserializes as the baseline file.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ScorecardMetrics {
    pub turns: usize,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_read_tokens: u64,
    pub total_cost_usd: f64,
    pub total_cost_cny: f64,
    /// `cache_read / (input + cache_read)`; `0.0` when there are no input
    /// tokens. Higher is better (more of the prompt was served from cache).
    pub cache_hit_ratio: f64,
}

/// A metric that grew beyond the allowed threshold versus the baseline.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Regression {
    pub metric: String,
    pub baseline: f64,
    pub current: f64,
    /// Percent increase over baseline. `f64::INFINITY` when baseline was 0.
    pub pct_increase: f64,
}

/// Full scorecard: per-turn breakdown plus aggregates.
#[derive(Debug, Clone, Serialize)]
pub struct Scorecard {
    pub per_turn: Vec<TurnScore>,
    pub metrics: ScorecardMetrics,
}

/// One row of input to the scorecard: a turn id, the model that served it, and
/// the turn's recorded usage.
pub struct TurnInput<'a> {
    pub turn_id: String,
    pub model: String,
    pub usage: &'a Usage,
}

/// A recorded turn as read from a scorecard input file (a JSON array of these).
/// Matches the per-turn data the `TurnEnd` hook already emits (`model` + `usage`),
/// so a run's turns can be captured and scored offline.
#[derive(Debug, Clone, Deserialize)]
pub struct RecordedTurn {
    #[serde(default)]
    pub turn_id: String,
    pub model: String,
    pub usage: Usage,
}

impl Scorecard {
    /// Build a scorecard from recorded per-turn usage. Pure + offline; cost is
    /// computed via the shared pricing layer (`None` pricing → unpriced, 0 cost).
    #[must_use]
    pub fn from_turns(turns: &[TurnInput<'_>]) -> Self {
        let mut per_turn = Vec::with_capacity(turns.len());
        let mut metrics = ScorecardMetrics::default();

        for turn in turns {
            // Normalize provider usage into canonical billable classes once.
            let classes = token_usage_for_pricing(turn.usage);
            let cost = calculate_turn_cost_estimate_from_usage(&turn.model, turn.usage);
            let (cost_usd, cost_cny, cost_unpriced) = match cost {
                Some(c) => (c.usd, c.cny, false),
                None => (0.0, 0.0, true),
            };

            metrics.turns += 1;
            metrics.total_input_tokens += classes.input;
            metrics.total_output_tokens += classes.output;
            metrics.total_cache_read_tokens += classes.cache_read;
            metrics.total_cost_usd += cost_usd;
            metrics.total_cost_cny += cost_cny;

            per_turn.push(TurnScore {
                turn_id: turn.turn_id.clone(),
                model: turn.model.clone(),
                input_tokens: classes.input,
                output_tokens: classes.output,
                cache_read_tokens: classes.cache_read,
                cost_usd,
                cost_cny,
                cost_unpriced,
            });
        }

        let cacheable = metrics.total_input_tokens + metrics.total_cache_read_tokens;
        metrics.cache_hit_ratio = if cacheable > 0 {
            metrics.total_cache_read_tokens as f64 / cacheable as f64
        } else {
            0.0
        };

        Self { per_turn, metrics }
    }

    /// Render a compact human-readable summary (used for non-JSON output).
    #[must_use]
    pub fn to_summary(&self) -> String {
        let m = &self.metrics;
        let unpriced = self.per_turn.iter().filter(|t| t.cost_unpriced).count();
        let mut out = String::new();
        out.push_str("Token / cache / cost scorecard\n");
        out.push_str(&format!("turns: {}\n", m.turns));
        out.push_str(&format!(
            "input_tokens: {}  output_tokens: {}  cache_read_tokens: {}\n",
            m.total_input_tokens, m.total_output_tokens, m.total_cache_read_tokens
        ));
        out.push_str(&format!(
            "cache_hit_ratio: {:.1}%\n",
            m.cache_hit_ratio * 100.0
        ));
        out.push_str(&format!(
            "cost_usd: ${:.4}  cost_cny: ¥{:.4}\n",
            m.total_cost_usd, m.total_cost_cny
        ));
        if unpriced > 0 {
            out.push_str(&format!(
                "note: {unpriced} turn(s) had no pricing row; their cost is excluded.\n"
            ));
        }
        out
    }
}

impl ScorecardMetrics {
    /// Flag metrics that grew more than `threshold_pct` over `baseline`. Cost
    /// and token counts are "lower is better", so only *increases* are
    /// regressions. (Cache-hit ratio is the opposite, reported separately.)
    #[must_use]
    pub fn regressions_against(
        &self,
        baseline: &ScorecardMetrics,
        threshold_pct: f64,
    ) -> Vec<Regression> {
        let mut out = Vec::new();
        push_regression(
            &mut out,
            "total_cost_usd",
            baseline.total_cost_usd,
            self.total_cost_usd,
            threshold_pct,
        );
        push_regression(
            &mut out,
            "total_input_tokens",
            baseline.total_input_tokens as f64,
            self.total_input_tokens as f64,
            threshold_pct,
        );
        push_regression(
            &mut out,
            "total_output_tokens",
            baseline.total_output_tokens as f64,
            self.total_output_tokens as f64,
            threshold_pct,
        );
        // Cache-hit ratio regresses when it *drops*; express the drop as a
        // positive percentage so it reads like the others.
        if baseline.cache_hit_ratio > 0.0 {
            let drop_pct = (baseline.cache_hit_ratio - self.cache_hit_ratio)
                / baseline.cache_hit_ratio
                * 100.0;
            if drop_pct > threshold_pct {
                out.push(Regression {
                    metric: "cache_hit_ratio_drop".to_string(),
                    baseline: baseline.cache_hit_ratio,
                    current: self.cache_hit_ratio,
                    pct_increase: drop_pct,
                });
            }
        }
        out
    }
}

fn push_regression(
    out: &mut Vec<Regression>,
    metric: &str,
    base: f64,
    cur: f64,
    threshold_pct: f64,
) {
    if base > 0.0 {
        let pct = (cur - base) / base * 100.0;
        if pct > threshold_pct {
            out.push(Regression {
                metric: metric.to_string(),
                baseline: base,
                current: cur,
                pct_increase: pct,
            });
        }
    } else if cur > 0.0 {
        out.push(Regression {
            metric: metric.to_string(),
            baseline: base,
            current: cur,
            pct_increase: f64::INFINITY,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usage(input: u32, output: u32, cache_hit: u32) -> Usage {
        Usage {
            input_tokens: input,
            output_tokens: output,
            prompt_cache_hit_tokens: Some(cache_hit),
            ..Default::default()
        }
    }

    #[test]
    fn aggregates_tokens_and_cache_hit_ratio_independent_of_pricing() {
        // input_tokens includes cache hits; token_usage_for_pricing splits them:
        // non-cached input = 1000-200 = 800, cache_read = 200.
        let u1 = usage(1000, 500, 200);
        let u2 = usage(2000, 100, 800); // non-cached = 1200, cache_read = 800
        let turns = [
            TurnInput {
                turn_id: "t1".into(),
                model: "unpriced-x".into(),
                usage: &u1,
            },
            TurnInput {
                turn_id: "t2".into(),
                model: "unpriced-x".into(),
                usage: &u2,
            },
        ];
        let card = Scorecard::from_turns(&turns);

        assert_eq!(card.metrics.turns, 2);
        assert_eq!(card.metrics.total_input_tokens, 800 + 1200);
        assert_eq!(card.metrics.total_output_tokens, 600); // 500 + 100
        assert_eq!(card.metrics.total_cache_read_tokens, 1000); // 200 + 800
        // cache_read / (input + cache_read) = 1000 / (2000 + 1000)
        let expected = 1000.0 / 3000.0;
        assert!((card.metrics.cache_hit_ratio - expected).abs() < 1e-9);
    }

    #[test]
    fn unknown_model_is_marked_unpriced_with_zero_cost() {
        let u = usage(1000, 500, 0);
        let turns = [TurnInput {
            turn_id: "t1".into(),
            model: "definitely-not-a-real-model".into(),
            usage: &u,
        }];
        let card = Scorecard::from_turns(&turns);
        assert!(card.per_turn[0].cost_unpriced);
        assert_eq!(card.per_turn[0].cost_usd, 0.0);
        assert_eq!(card.metrics.total_cost_usd, 0.0);
        assert!(card.to_summary().contains("no pricing row"));
    }

    #[test]
    fn regression_flags_cost_and_token_increases_over_threshold() {
        let baseline = ScorecardMetrics {
            turns: 1,
            total_input_tokens: 1000,
            total_output_tokens: 1000,
            total_cache_read_tokens: 0,
            total_cost_usd: 0.10,
            total_cost_cny: 0.7,
            cache_hit_ratio: 0.5,
        };
        let current = ScorecardMetrics {
            total_cost_usd: 0.20,      // +100% → regression
            total_input_tokens: 1010,  // +1% → under 5% threshold, no regression
            total_output_tokens: 2000, // +100% → regression
            cache_hit_ratio: 0.5,      // unchanged
            ..baseline.clone()
        };
        let regs = current.regressions_against(&baseline, 5.0);
        let names: Vec<&str> = regs.iter().map(|r| r.metric.as_str()).collect();
        assert!(names.contains(&"total_cost_usd"));
        assert!(names.contains(&"total_output_tokens"));
        assert!(!names.contains(&"total_input_tokens")); // under threshold
    }

    #[test]
    fn regression_flags_cache_hit_ratio_drop() {
        let baseline = ScorecardMetrics {
            cache_hit_ratio: 0.80,
            ..Default::default()
        };
        let current = ScorecardMetrics {
            cache_hit_ratio: 0.40,
            ..Default::default()
        };
        let regs = current.regressions_against(&baseline, 10.0);
        assert!(regs.iter().any(|r| r.metric == "cache_hit_ratio_drop"));
    }

    #[test]
    fn no_regressions_when_within_threshold() {
        let baseline = ScorecardMetrics {
            total_cost_usd: 1.0,
            total_input_tokens: 1000,
            total_output_tokens: 1000,
            cache_hit_ratio: 0.5,
            ..Default::default()
        };
        let current = baseline.clone();
        assert!(current.regressions_against(&baseline, 5.0).is_empty());
    }
}
