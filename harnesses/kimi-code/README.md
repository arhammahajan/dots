# Kimi Code

`config.toml` mirrors this computer's Kimi Code settings: the OpenCode Go
provider at `https://opencode.ai/inference/go/openai/v1` with every model from
the models.dev catalog, DeepSeek V4.1 Flash as the default, and the CLI's
model and context metadata.

The provider reads its key from `OPENCODE_API_KEY` through `api_key_env`, so no
secret lives in this repo. `.zshenv` exports that variable from
`~/.config/ai/credentials.env`; rotate the key in that file only.

OpenCode Go rejects requests that omit an `x-opencode-session` header, so
`custom_headers` pins a stable per-install UUID. Muse Code generates one per
conversation in `.zshenv`, but Kimi's config is static, so a single install
identifier covers every session.

To link the config on another machine, back up any existing config first,
then run from the dotfiles checkout:

```sh
kimi_dir="${KIMI_CODE_HOME:-$HOME/.kimi-code}"
mkdir -p "$kimi_dir"
ln -s "$PWD/harnesses/kimi-code/config.toml" "$kimi_dir/config.toml"
```

Only link `config.toml`. Device identity, region, sessions, caches, telemetry,
workspace trust, `tui.toml` (still at defaults), and the CLI binary stay
outside the repo.
