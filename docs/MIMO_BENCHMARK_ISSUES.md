# MiMo v2.5 Benchmarking — Known Issues

Tracking doc for quirks and workarounds when benchmarking Xiaomi MiMo v2.5
through DeepSeek's harness integrations.

## PinchBench

### Issue 1: Model validation requires OpenRouter prefix

PinchBench validates models against OpenRouter's `/models` endpoint. If you
pass `mimo-v2.5-pro` without the `openrouter/xiaomi/` prefix, validation is
skipped entirely (it assumes it's a non-OpenRouter model). This means you
won't know if the model ID is wrong until the run fails.

**Workaround:** Always use `openrouter/xiaomi/mimo-v2.5-pro` for OpenRouter
routing, or use `--direct-mimo` for the Xiaomi API.

### Issue 2: PinchBench requires OPENROUTER_API_KEY

Even when using a direct provider, PinchBench's `lib_agent.py` checks for
`OPENROUTER_API_KEY` in some code paths. The `--direct-mimo` flag in our
runner works around this by setting up a custom OpenAI-compatible provider
entry in OpenClaw's `models.json` and exporting `OPENAI_API_KEY`/`OPENAI_BASE_URL`.

### Issue 3: Token Plan vs Pay-as-you-go key mismatch

Xiaomi MiMo has two API endpoints:
- **Token Plan** (`tp-` keys): `https://token-plan-sgp.xiaomimimo.com/v1`
- **Pay-as-you-go** (`sk-` keys): `https://api.xiaomimimo.com/v1`

Using the wrong key type with the wrong endpoint produces auth errors. The
runner now detects this and warns.

### Issue 4: OpenClaw is the runtime, not DeepSeek

PinchBench runs tasks through OpenClaw, not DeepSeek. This means the
benchmark measures MiMo v2.5's performance through OpenClaw's agent harness,
not through DeepSeek's tool system. For DeepSeek-native evaluation,
Terminal-Bench (via Harbor) is the better fit.

**Future:** Create a DeepSeek-native PinchBench adapter that loads tasks
from PinchBench's `tasks/` directory and runs them through `deepseek exec`.

## Terminal-Bench (Harbor)

### Issue 1: MiMo provider routing

Harbor passes models as `provider/model` format. For MiMo via OpenRouter,
use `openrouter/xiaomi/mimo-v2.5-pro`. For direct Xiaomi API, pass
`--provider xiaomi-mimo` as an extra agent flag.

### Issue 2: Container environment

The Harbor adapter installs deepseek via npm in the container. MiMo API
keys must be forwarded from the host environment. The adapter checks for
`XIAOMI_MIMO_API_KEY`, `OPENROUTER_API_KEY`, and `OPENAI_API_KEY`.

## SWE-bench

### Issue 1: MiMo thinking mode

MiMo v2.5 Pro supports extended thinking. For SWE-bench patch generation,
ensure the thinking level is set appropriately. The `--thinking high` flag
is passed through the CLI.

### Issue 2: Context window

MiMo v2.5 Pro has a 128K context window. Large SWE-bench instances (e.g.,
Django, sympy) may benefit from the full window. No special handling needed,
but worth monitoring token usage.

## Environment Variables Reference

```
# Xiaomi MiMo direct API
XIAOMI_MIMO_API_KEY=tp-...    # Token Plan key
XIAOMI_MIMO_API_KEY=sk-...    # Pay-as-you-go key
XIAOMI_MIMO_BASE_URL=https://token-plan-sgp.xiaomimimo.com/v1
XIAOMI_MIMO_MODEL=mimo-v2.5-pro

# Aliases also accepted
XIAOMI_API_KEY=...
MIMO_API_KEY=...
MIMO_BASE_URL=...
MIMO_MODEL=...

# OpenRouter (for MiMo via OpenRouter)
OPENROUTER_API_KEY=...
```
