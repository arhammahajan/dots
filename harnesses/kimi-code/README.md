# Kimi Code through model-router

`config.toml`, linked at `~/.kimi-code/config.toml`, defines Chat, Responses,
and Anthropic providers against the central router. The default is
`model-router/deepseek-v4.1-flash`. Provider credentials come from the public
`MODEL_ROUTER_API_KEY=unused` client setting, sourced by `.zshenv`.

Update the tracked model entries manually from the router metadata, preserving
context, output, modality, and effort settings. Validate with `kimi doctor`.
The legacy refresh script is disabled because it discards capability settings. See the [shared setup](../README.md).
