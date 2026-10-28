# Muse through model-router

Muse connects directly to `http://127.0.0.1:4000/v1`. Model-router serves Muse's
catalog at `/muse-code/models` and `/v1/muse-code/models`, including display labels
and context/output limits from its central metadata. Generation uses the router's
existing `/v1/responses` endpoint with its normal streaming and session handling.
There is no Muse adapter process or service.

`/models` lists every router model. The router translates Responses
requests for other upstream protocols, reports nonportable controls through
`x-model-router-loss`, and preserves Muse's namespaced function tools through
translation, including streamed tool calls.

Use `~/.local/bin/muse-with-env` as T3's Muse binary. It sets `META_API_KEY` from
the router client environment after T3 launches it; no Meta login or upstream
key is needed. Normal Zsh sessions load the same settings from `.zshenv`.

```sh
systemctl --user status model-router
curl http://127.0.0.1:4000/v1/muse-code/models
```

See the [shared setup](../README.md) for installation and model refresh.
