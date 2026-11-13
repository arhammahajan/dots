# Kimi Code through model-router

`config.toml`, copied to `~/.kimi-code/config.toml`, defines Chat, Responses,
and Anthropic providers against the central router. Provider credentials come
from the public `MODEL_ROUTER_API_KEY=unused` client setting, sourced by `.zshenv`.

Update the tracked model entries manually from the router metadata, preserving
context, output, modality, and effort settings. Validate with `kimi doctor`.
See the [shared setup](../README.md) for services and configuration paths.
