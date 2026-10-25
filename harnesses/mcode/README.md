# MiniMax Code

`config.yaml` mirrors this computer's MiniMax Code (`mcode`) settings: DeepSeek
V4.1 Flash as the default model and every working OpenCode Go model exposed
through three custom providers, one per wire protocol.

## Why three providers

OpenCode Go serves different model families on different APIs, and mcode fixes
the API format per provider rather than per model:

| Provider | `api` | Base URL | Models |
| -------- | ----- | -------- | ------ |
| `opencode-go` | `openai-completions` | `https://opencode.ai/inference/go/openai/v1` | 21 chat-completions models |
| `opencode-go-responses` | `openai-responses` | `https://opencode.ai/inference/go/openai/v1` | GPT 5.6/6 Luna, Grok 4.7, two Muse Spark contributors, LongCat 2.5 Preview Free, Iceman Contributor Free |
| `opencode-go-anthropic` | `anthropic-messages` | `https://opencode.ai/inference/go/anthropic` | Claude Haiku 5.5, MiniMax M2.7 |

`mcode` appends `/chat/completions`, `/responses`, or `/v1/messages` to the base
URL, so these three bases produce OpenCode Go's documented endpoints. Models on
Google's Gemini wire (`gemini-4-argon`) cannot work at all, because mcode has no
Gemini API format. Seven of the 37 catalog models are left out: that Gemini
model, four that upstream reports as unavailable or unsupported
(`cat-jam-free`, `mimo-v2.5-pro`, `space-bunny`, `step-5-preview-free`), and
two that upstream has retired (`gemini-4-argon-contributor`,
`minimax-m3.1-flash-free`). Every other model was checked with
`mcode provider test`.

The default model is written fully qualified:

```yaml
defaultModel: custom_provider:opencode-go/deepseek-v4.1-flash
```

A bare `opencode-go/deepseek-v4.1-flash` resolves to the built-in
`configured_provider` route and fails with `DEFAULT_MODEL_NOT_AVAILABLE_FOR_ROUTE`.

All three providers send an `x-opencode-session` header, which OpenCode Go
requires on every request; the value is a stable per-install UUID.

The two Anthropic-wire models also declare `variants` that switch thinking to
`adaptive`. Without them mcode sends `thinking.type: enabled`, which OpenCode
Go's Anthropic endpoint rejects with `invalid_request_error`.

## No symlink, and no key in this repo

Unlike the other harnesses here, `~/.minimax/config.yaml` is a **separate copy**
rather than a symlink to this file. mcode stores custom-provider keys as literal
values and offers no environment-variable reference for them, so this public
repo carries `PLACEHOLDER_OPENCODE_API_KEY` instead of the real key.

To set up a machine, run from the dotfiles checkout:

```sh
mkdir -p ~/.minimax
cp harnesses/mcode/config.yaml ~/.minimax/config.yaml
```

Then edit `~/.minimax/config.yaml` and replace all three
`PLACEHOLDER_OPENCODE_API_KEY` values with the key from `OPENCODE_API_KEY` in
`~/.config/ai/credentials.env`, and restrict the file:

```sh
chmod 600 ~/.minimax/config.yaml
```

Re-copy by hand when this file changes. Keep the live file out of Git.

## T3 Code

`~/.t3/userdata/settings.json` registers mcode over the Agent Client Protocol as
`acpRegistry_mcode` (`/home/arhammahajan/.minimax-code/bin/mcode acp`), the same
shape as the Kimi Code entry. MiniMax login state, sessions, caches, plugins,
and telemetry identity stay in `~/.minimax/` and `~/.minimax-code/` outside the
repo.
