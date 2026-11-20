# Antigravity model capabilities

This reference lists the 28 custom models in `harnesses/antigravity/cli/settings.json` and the router metadata queried from the local `model-router` `/v1/models` endpoint on 2026-10-28. See [Antigravity CLI setup](README.md) for launcher and provider configuration.

Each entry in `customModelsConfig.customModels` sets `maxTokens` to the router context window. All 28 entries set `supportsToolCalls` and `supportsThinking` to `true`, while 20 entries set `supportsImages` to `true`. For `local-llama`, the `128,000` token limit reflects the local server allocation at the time of the snapshot. Query `/v1/models` on `model-router` to check the current runtime limit.

The router efforts column lists the reasoning effort values that `model-router` reports for reference. The Antigravity CLI does not accept custom-model effort lists or `--effort` overrides in `customModelsConfig`, so these values do not appear in the CLI interface. Models without published effort values appear as `Unknown`, and the configuration omits any reasoning default.

## Configured models

| Model | Context tokens | Image input | Router efforts |
| --- | ---: | --- | --- |
| `local-llama` | 128,000 | No | none, minimal, low, medium, high, xhigh |
| `deepseek-v4.1-flash` | 1,000,000 | Yes | low, high, max |
| `deepseek-v4-pro` | 1,000,000 | No | high, max |
| `glm-5.2` | 1,000,000 | No | high, max |
| `glm-5.3` | 1,000,000 | No | low, high, max |
| `grok-4.7` | 500,000 | Yes | low, medium, high, xhigh |
| `muse-spark-1.2-contributor` | 1,048,576 | Yes | minimal, low, medium, high, xhigh |
| `muse-spark-1.3-contributor` | 1,048,576 | Yes | minimal, low, medium, high, xhigh |
| `glm-5.3-flash` | 1,000,000 | Yes | low, high, max |
| `gpt-5.6-luna` | 1,050,000 | Yes | none, low, medium, high, xhigh, max |
| `gpt-6-luna` | 1,050,000 | Yes | none, low, medium, high, xhigh, max |
| `claude-haiku-5-5` | 1,000,000 | Yes | low, medium, high, xhigh, max |
| `hy3` | 256,000 | No | none, low, high |
| `hy4-preview` | 1,024,000 | No | none, high |
| `kimi-k2.7-code` | 262,144 | Yes | Unknown |
| `kimi-k3` | 1,048,576 | Yes | max |
| `mimo-v2.6-flash` | 1,048,576 | Yes | Unknown |
| `mimo-v2.6-pro` | 1,048,576 | Yes | Unknown |
| `minimax-m3` | 1,000,000 | Yes | Unknown |
| `minimax-m3.1-flash` | 1,000,000 | Yes | low, medium, high, xhigh, max |
| `longcat-2.0` | 1,000,000 | No | Unknown |
| `step-5-preview` | 1,000,000 | Yes | low, medium, high |
| `beam-free` | 1,048,576 | No | low, medium, high, xhigh, max |
| `qwen3.8-max` | 1,000,000 | Yes | low, medium, xhigh |
| `qwen3.8-flash` | 1,000,000 | Yes | low, medium, xhigh |
| `qwen3.7-plus` | 1,000,000 | Yes | Unknown |
| `kimi-k3.1-pro` | 1,048,576 | Yes | low, high, max |
| `gemini-4-argon` | 1,048,576 | Yes | low, medium, high |
