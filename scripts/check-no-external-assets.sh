#!/usr/bin/env bash
# Guard against render-blocking external assets in the built frontend.
#
# The app is a desktop tool: everything it needs to draw its window must ship
# inside the bundle. A stylesheet or font fetched from the network is
# render-blocking, and on a connection that black-holes the host (DPI filtering,
# a dead proxy, no internet) the request never resolves and the window stays
# blank white instead of showing the UI.

set -euo pipefail

DIST="${1:-youtube-downloader/dist}"

if [ ! -f "$DIST/index.html" ]; then
  echo "❌ $DIST/index.html not found - build the frontend first."
  exit 1
fi

# Only <link>/@import/url() references matter: they are fetched to render the
# page. Remote URLs inside JS strings (API endpoints, proxy examples) are fine.
found=$(grep -oE '(<link[^>]+href|@import[[:space:]]+|url\()[^>]*https?://[a-zA-Z0-9.-]+' \
  "$DIST/index.html" "$DIST"/assets/*.css 2>/dev/null \
  | grep -v 'http://www\.w3\.org' || true)

if [ -n "$found" ]; then
  echo "❌ The built UI still loads assets over the network:"
  echo "$found" | sed 's/^/   /'
  echo ""
  echo "   Bundle them instead (see @fontsource-variable/inter in src/styles.css)."
  echo "   Otherwise the app shows a blank white window whenever the host is unreachable."
  exit 1
fi

echo "✓ No external assets in the built UI - the window renders offline."
