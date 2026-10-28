# Antigravity model capabilities

Source: local model-router `/v1/models`, checked 2026-10-28.

`maxTokens` in the custom-model settings uses the router context window.
The effort column records router capabilities for reference. Antigravity does
not accept these enums for custom models; they are not configured UI choices.
Unknown effort enums are left unknown. No reasoning default is invented.

| Model | Context tokens | Router efforts |
| --- | ---: | --- |
| `local` | 128,000 | none, minimal, low, medium, high, xhigh |
| `deepseek-v4.1-flash` | 1,000,000 | low, high, max |
| `deepseek-v4-pro` | 1,000,000 | high, max |
| `glm-5.2` | 1,000,000 | high, max |
| `glm-5.3` | 1,000,000 | low, high, max |
| `grok-4.7` | 500,000 | low, medium, high, xhigh |
| `muse-spark-1.2-contributor` | 1,048,576 | minimal, low, medium, high, xhigh |
| `muse-spark-1.3-contributor` | 1,048,576 | minimal, low, medium, high, xhigh |
| `glm-5.3-flash` | 1,000,000 | low, high, max |
| `gpt-5.6-luna` | 1,050,000 | none, low, medium, high, xhigh, max |
| `gpt-6-luna` | 1,050,000 | none, low, medium, high, xhigh, max |
| `claude-haiku-5-5` | 1,000,000 | low, medium, high, xhigh, max |
| `hy3` | 256,000 | none, low, high |
| `hy4-preview` | 1,024,000 | none, high |
| `kimi-k2.7-code` | 262,144 | Unknown |
| `kimi-k3` | 1,048,576 | max |
| `mimo-v2.5` | 1,000,000 | Unknown |
| `mimo-v2.6-flash` | 1,048,576 | Unknown |
| `mimo-v2.5-pro` | 1,048,576 | Unknown |
| `mimo-v2.6-pro` | 1,048,576 | Unknown |
| `minimax-m3` | 1,000,000 | Unknown |
| `minimax-m3.1-flash` | 1,000,000 | low, medium, high, xhigh, max |
| `longcat-2.0` | 1,000,000 | Unknown |
| `step-5-preview` | 1,000,000 | low, medium, high |
| `beam-free` | 1,048,576 | low, medium, high, xhigh, max |
| `qwen3.8-max` | 1,000,000 | low, medium, xhigh |
| `qwen3.8-flash` | 1,000,000 | low, medium, xhigh |
| `qwen3.7-plus` | 1,000,000 | Unknown |
| `kimi-k3.1-pro` | 1,048,576 | low, high, max |
| `gemini-4-argon` | 1,048,576 | low, medium, high |
