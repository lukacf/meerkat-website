#!/bin/bash
# Build WASM demos for embedding on rkat.ai.
#
# Uses @rkat/web npm package for the WASM runtime (no Rust toolchain needed).
# Builds demo Vite apps from the meerkat repo examples.
#
# The demos support a ?proxy=URL query parameter for server-provided API keys.
# When ?proxy= is set, the demos auto-hide API key UI, fill dummy keys, and
# route LLM calls through the proxy ({proxy}/anthropic, /openai, /gemini).
# The Axum wrapper pages set ?proxy= via the PROXY_URL env var.
#
# Usage:
#   ./scripts/build-demos.sh                          # defaults
#   RKAT_WEB_VERSION=0.4.3 ./scripts/build-demos.sh   # pin @rkat/web version
#   RAIK_DIR=/path/to/raik ./scripts/build-demos.sh    # custom raik repo path

set -euo pipefail

RKAT_WEB_VERSION="${RKAT_WEB_VERSION:-0.4.3}"  # @rkat/web npm package (WASM runtime + proxy)
RAIK_DIR="${RAIK_DIR:-$HOME/src/raik}"         # meerkat repo (demo example source code)
WEBSITE_DIR="$(cd "$(dirname "$0")/.." && pwd)"

if [[ ! -d "$RAIK_DIR/examples/031-wasm-mini-diplomacy-sh" ]]; then
  echo "Error: $RAIK_DIR doesn't have the demo examples" >&2
  exit 1
fi

# ── Install @rkat/web (includes pre-built WASM) ──────────────────

echo "=== Installing @rkat/web@${RKAT_WEB_VERSION} ==="
cd "$WEBSITE_DIR"
npm install "@rkat/web@${RKAT_WEB_VERSION}" --save-exact
WASM_PKG="$WEBSITE_DIR/node_modules/@rkat/web/wasm"

if [[ ! -f "$WASM_PKG/meerkat_web_runtime_bg.wasm" ]]; then
  echo "Error: WASM not found in @rkat/web package. Check package contents." >&2
  ls "$WEBSITE_DIR/node_modules/@rkat/web/" >&2
  exit 1
fi

echo "WASM runtime: @rkat/web@${RKAT_WEB_VERSION}"

# ── Diplomacy demo ────────────────────────────────────────────────

echo "=== Building diplomacy demo ==="
cd "$RAIK_DIR/examples/031-wasm-mini-diplomacy-sh/web"
npm install --silent

# Dummy keys — just need to be non-empty so all providers are "available".
# Real auth happens via the proxy when ?proxy= is set.
VITE_ANTHROPIC_API_KEY='placeholder' \
VITE_OPENAI_API_KEY='placeholder' \
VITE_GEMINI_API_KEY='placeholder' \
  npm run build

DIPLO="$WEBSITE_DIR/demos/diplomacy/app"
rm -rf "$DIPLO"
mkdir -p "$DIPLO"
cp -r dist/assets "$DIPLO/"
cp "$WASM_PKG/meerkat_web_runtime.js" "$DIPLO/runtime.js"
cp "$WASM_PKG/meerkat_web_runtime_bg.wasm" "$DIPLO/meerkat_web_runtime_bg.wasm"

# Write index.html with relative asset paths
DIPLO_JS=$(basename "$DIPLO/assets/"index-*.js)
DIPLO_CSS=$(basename "$DIPLO/assets/"index-*.css)
cat > "$DIPLO/index.html" << HTML
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Mini Diplomacy Arena — Meerkat WASM Demo</title>
    <link rel="preconnect" href="https://fonts.googleapis.com" />
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
    <link href="https://fonts.googleapis.com/css2?family=Chakra+Petch:wght@400;500;600;700&family=Playfair+Display:ital,wght@0,400;0,700;1,400&family=Space+Grotesk:wght@400;500;600;700&display=swap" rel="stylesheet" />
    <style>
    /* Hide settings gear — models are server-controlled on rkat.ai */
    .gear-btn, .settings-drawer { display: none !important; }
    </style>
    <script type="module" crossorigin src="./assets/${DIPLO_JS}"></script>
    <link rel="stylesheet" crossorigin href="./assets/${DIPLO_CSS}">
  </head>
  <body>
    <div id="app"></div>
  </body>
</html>
HTML

echo "Diplomacy: $DIPLO"

# ── WebCM demo ────────────────────────────────────────────────────

echo "=== Building WebCM demo ==="
cd "$RAIK_DIR/examples/032-wasm-webcm-agent/web"
npm install --silent

ANTHROPIC_API_KEY='placeholder' \
OPENAI_API_KEY='placeholder' \
GEMINI_API_KEY='placeholder' \
  npm run build

WEBCM="$WEBSITE_DIR/demos/webcm/app"
rm -rf "$WEBCM"
mkdir -p "$WEBCM"
cp -r dist/* "$WEBCM/"
cp -r public/* "$WEBCM/" 2>/dev/null || true

# Replace meerkat WASM with the npm package version (matched pair)
mkdir -p "$WEBCM/meerkat-pkg"
cp "$WASM_PKG/meerkat_web_runtime.js" "$WEBCM/meerkat-pkg/meerkat_web_runtime.js"
cp "$WASM_PKG/meerkat_web_runtime_bg.wasm" "$WEBCM/meerkat-pkg/meerkat_web_runtime_bg.wasm"

# Fix absolute URLs to relative (Vite builds with base="/")
WEBCM_BUNDLE="$WEBCM/assets/"index-*.js
sed -i '' 's|new URL("/webcm.mjs"|new URL("./webcm.mjs"|g; s|new URL("/meerkat-pkg/|new URL("./meerkat-pkg/|g' $WEBCM_BUNDLE

# Copy Vite-generated index.html and fix asset paths to relative
sed -i '' 's|href="/src/|href="./src/|g; s|src="/src/|src="./src/|g; s|href="/assets/|href="./assets/|g; s|src="/assets/|src="./assets/|g' "$WEBCM/index.html"

echo "WebCM: $WEBCM"

# ── Done ──────────────────────────────────────────────────────────

echo ""
echo "=== Build complete (WASM from @rkat/web@${RKAT_WEB_VERSION}) ==="
echo ""
echo "To run locally:"
echo "  ANTHROPIC_API_KEY=sk-... OPENAI_API_KEY=sk-... GEMINI_API_KEY=AI... cargo run"
echo "  open http://localhost:3001/demos/diplomacy/"
echo ""
echo "Axum proxies LLM calls directly — no separate proxy process needed."
