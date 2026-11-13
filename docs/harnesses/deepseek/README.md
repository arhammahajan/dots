# DeepSeek Harness through model-router

The published [DeepSeek Harness](https://github.com/deepseek-ai/deepseek-harness)
`@deepseek-ai/dsh@0.2.1-rc.5` is installed under
`~/.local/share/deepseek-harness`. It requires Node >=24.9.0; this machine uses
fnm's Node 24.22.1. The launcher also finds fnm's default Node when started
outside an interactive shell.

Use `harness deepseek` (or the interactive Zsh alias `dsh`). The central launcher
sources the shared `~/.config/ai/model-router.env` and uses
the public `MODEL_ROUTER_API_KEY=unused`. Only model-router holds upstream keys.
The provider connects directly to `http://127.0.0.1:4000/v1`.

```sh
dsh                                      # start the Web UI and open a browser
dsh web --no-open                         # print the authenticated local URL
dsh headless "explain this repository"    # one task in the current directory
dsh acp                                  # ACP over stdio for compatible clients
```

The Web UI listens on loopback port 3080. Use the URL printed by the launcher:
it includes a generated access token that establishes the browser cookie.
An unauthenticated request to the bare URL returns 401. Stop the server with
Ctrl-C. No additional persistent service or compatibility proxy is required.
The existing model-router user service must be running.

## Models and configuration

`cordis.patch.yml` is copied into `~/.dsh/profiles/<profile>/cordis.patch.yml`
for `web`, `headless`, `acp`, `sdk`, and `sdk-minimal`. These are managed copies
because the Models settings page writes its own profile patch. The default is
**DeepSeek V4.1 Flash**, with **low** reasoning effort, matching the Pi setup.
ACP additionally appends `acp.patch.yml` to override that entry point's explicit
provider/model fallback; its initial effort is **Provider default** until the
client selects an effort.
Select another model and effort in the Web UI's model picker. The saved default
belongs to that profile; existing conversations retain their own selection.

The snapshot contains all **27** models advertised by `/v1/models` on
2026-11-04. Context windows, output limits, image capability, and known effort
lists come from `metadata.router`. No local llama.cpp model was advertised at
setup time. Unknown effort lists are omitted, keeping the destination's default;
the harness only represents text and image input modalities. Router `none`
effort is shown as `off` and mapped back to `none` on the wire.

All models use **Responses ingress**. The router translates to Chat,
Anthropic, or Gemini where needed, just as it does for Muse. DeepSeek's
Responses adapter sends its durable session ID as `session_id` and
`prompt_cache_key`; model-router uses this identity for upstream affinity.
There is no shared install-wide or process-wide session ID. This also avoids
the installed adapter's inability to configure native Chat/Anthropic affinity
headers for a custom provider. Nonportable Responses controls use the router's
existing lossy compatibility policy.

Model discovery in **Settings → Models → Fetch available models** can add new
IDs, but does not replace the router's richer capability metadata. Update the
tracked entries and merge them into active profile copies when the router
inventory changes, preserving personal model choices and settings. Additional
profiles need the same provider patch; an unconfigured shipped profile retains
DeepSeek's official-provider defaults.

## Reinstall

With the shared router already configured:

```sh
~/dev/dots/harnesses/deepseek/setup.sh
```

The helper installs the pinned release in an isolated npm prefix, initializes
the five profiles, copies the router template, and links the central launcher.
Existing different profile patches and launcher destinations are backed up under
`~/.local/share/dots-backups/deepseek-<timestamp>/`. Rerunning resets those
profiles to the tracked template; merge personal settings from the backups.
Required native dependency install scripts are explicitly allowed for this
install without changing the user's global npm policy.

## T3 Code ACP provider

Select **DeepSeek Harness** in T3 Code's provider picker. Instance
`acpRegistry_dsh` runs `~/.local/bin/harness` with
`commandArgs: ["deepseek", "acp"]` as a local ACP command.
The launcher supplies the shared router environment and Node, so no provider
environment overrides, registry install, or DeepSeek login are needed.

`t3-provider.json` records the entry merged into `providerInstances` in
`~/.t3/userdata/settings.json`. Existing T3 settings were backed up alongside
that file before registration. For another machine, run `setup.sh`, adjust the
absolute command path, and merge that entry into T3's settings.

T3 discovers all 27 router models from the agent's ACP session, including their
display names. ACP model IDs encode the provider/model pair as a JSON array;
keep these IDs intact when configuring T3 programmatically. The agent reports
reasoning choices for the selected model through ACP config options. T3's
initial catalog can repeat the current model's choices for other models;
switching models lets the agent publish the appropriate options for that model.

Registration was verified against T3's live provider catalog without restarting
the app. Direct ACP verification covered initialization, exact router inventory
comparison, effort selection, a streamed response returning `DSH_T3_ACP_OK`,
and session close.

## Validation

Setup verified the version, composed profiles, and shell-independent launcher;
live requests through DeepSeek V4.1 Flash, Qwen 3.8 Flash, and Gemini 4 Argon;
a two-request DeepSeek shell tool round trip; and Web UI token/cookie access.
Inference was sampled across upstream protocols, not run against every model.

```sh
dsh --version
dsh web --dump-config
systemctl --user status model-router
```
