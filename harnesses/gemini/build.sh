#!/bin/sh
# Rebuild the maintained Gemini source checkout after rebasing.
set -eu

repo=${GEMINI_CLI_REPO:-$HOME/dev/gemini-cli}
case ${1:-} in
    "") install=false ;;
    --install) install=true ;;
    *) printf 'usage: %s [--install]\n' "$0" >&2; exit 2 ;;
esac

cd "$repo"
if [ "$install" = true ] || [ ! -d node_modules ]; then
    # Root prepare bundles the whole distribution; build the CLI explicitly below.
    npm ci --ignore-scripts
fi
npm run generate
npm run build --workspace @google/gemini-cli-core
npm run build --workspace @google/gemini-cli-devtools
npm run build --workspace @google/gemini-cli
