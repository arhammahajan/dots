#!/bin/sh
set -eu

# Upstream credentials stay in model-router; clients use its loopback key.
. "${XDG_CONFIG_HOME:-$HOME/.config}/ai/model-router.env"
exec cline auth --provider openai \
  --apikey "$MODEL_ROUTER_API_KEY" \
  --baseurl "${MODEL_ROUTER_URL%/}/v1" \
  --modelid "${1:-deepseek-v4.1-flash}"
