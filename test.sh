#!/bin/sh

set -v
set -e

# Use the default python spelling, if the user hasn't specified one
PYTHON=${PYTHON:-python}

# Test our shell script code portability and quality (including this script)
# SC1117 was disabled after 0.5, because it was too pedantic
EXCLUSIONS="--exclude=SC1117"
if command -v shellcheck; then
    find . -name "*.sh" -exec shellcheck "$EXCLUSIONS" {} +
fi

## moria1, Serge, no caches extra info, no dir port, doesn't exist
## TODO: validate output from all 3 commands using grep, grep, and stem?
$PYTHON generateFallbackDirLine.py 9695DFC35FFEB861329B9F1AB04C46397020CE31 BA44A889E64B93FAA2B114E02C2A279A8555C533 001524DD403D729F08F7E5D77813EF12756CFA8D 5AFAC3D00E97D6733112CC9CA2A788691FA87125 AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
$PYTHON lookupFallbackDirContact.py 9695DFC35FFEB861329B9F1AB04C46397020CE31 BA44A889E64B93FAA2B114E02C2A279A8555C533 001524DD403D729F08F7E5D77813EF12756CFA8D 5AFAC3D00E97D6733112CC9CA2A788691FA87125 AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA

## If we get the top 200 relays from Onionoo, we generate a list with about
## 10 fallbacks
export TOR_FB_ONIONOO_LIMIT=200
## Hide info-level logs
$PYTHON updateFallbackDirs.py 2>&1 | tee -a fallback.log | grep -v "INFO::"
