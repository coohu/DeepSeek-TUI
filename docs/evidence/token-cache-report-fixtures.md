# Token/cache report fixture matrix

This matrix supports #3390 by tying user token/cache reports to concrete
fixtures or next actions. It is intentionally evidence-focused: reports without
provider/model/transcript detail stay open for fresh data instead of being
treated as proven regressions.

## Fixture Coverage

| Source | Reporter / helper | Report shape | Fixture |
| --- | --- | --- | --- |
| #1177 | @douglarek | Five-turn `/cache` table with one low 56.8% tail turn and 86.0% aggregate hit ratio | `cache_command_replays_reported_1177_low_hit_fixture` |
| #1747 | @Amund | DeepSeek-TUI aggregate: 21,356,928 hit, 8,470,281 miss, 165,624 output, clearly below desired 90%+ cache hit target | `cache_stats_flags_reported_1747_low_hit_fixture` |

## Related Report Triage

| Issue | Disposition | Next action |
| --- | --- | --- |
| #743 | Needs fresh reporter data | Original report mixed high spend, old versions, UI freeze, and macOS color readability. Keep asking for current version, provider/model, dashboard token counts, and cache hit/miss totals. |
| #1120 | Repro direction identified | Use existing prompt-inspect/tool-catalog stability tests plus the #1177/#1747 cache fixtures. Further closure needs provider-request snapshots or paired same-task runs. |
| #1177 | Fixture linked | Use the five-turn table fixture as the reproducible cache-history smoke case; keep broader cache-architecture work open. |
| #1747 | Fixture linked | Use the aggregate low-hit stats fixture as the regression smoke case; exact first-differing-byte diagnosis still needs request snapshots. |
| #1818 | Needs reporter data | Screenshots show high daily spend but comments note 96.6-98.3% hit rate and no transcript/task detail. Ask for task shape, large-file reads, compaction state, and provider/model. |
| #1863 | Duplicate / moved | Already closed as duplicate of #3275 for model self-questioning loop behavior. Track there, not under cache-hit fixtures. |
| #2953 | Separate prompt-size lane | Needs before/after prompt-size comparison and prompt-layer audit; not closed by cache-history fixtures. |
| #2956 | Separate transcript-growth lane | Needs per-turn input growth analysis and benchmark/exec transcript fixtures; not closed by cache-history fixtures. |
| #2958 | Partially covered elsewhere | Prompt-mode matrix work exists in #3611 but remains policy-blocked from merging; leave issue open until that PR lands. |
| #2961 | Resolved prerequisite | Usage normalization was marked resolved for v0.8.65 via #3509/#3544, so #3390 can consume normalized cache telemetry instead of reparsing provider payloads. |

## Remaining Gaps

- Request-body snapshot fixtures are still needed for first-differing-byte
  diagnosis when a user reports a low hit rate with two consecutive requests.
- Benchmark-harness fixtures are still needed for #2956-style repeated
  transcript growth and large tool-output replay.
- Cost per completed task and quota-style telemetry remain outside this slice;
  they belong to provider usage/pricing display work.
