# MCode through model-router

`config.yaml`, copied to `~/.minimax/config.yaml`, uses three custom provider groups:
`model-router` (Chat), `model-router-responses`, and `model-router-anthropic`.

Every group points at `http://127.0.0.1:4000/v1` with the public placeholder
`unused`. There is no upstream key in the active config. The former private copy
has been backed up. MCode replaces symlinks when saving, so keep an active copy.

MCode sends its durable conversation ID as `X-Mavis-Session-Id`; model-router
maps it to the upstream's session header. Model limits and advertised reasoning
levels come from the router. `--effort` uses the levels reported for that model.

The file uses JSON syntax, which MCode's YAML reader accepts. Update model entries
manually from router metadata, preserving capability settings, then run
`cp harnesses/mcode/config.yaml ~/.minimax/config.yaml`.
Inspect the result with `mcode provider list --json`.
See the [shared setup](../README.md) for services and compatibility.
