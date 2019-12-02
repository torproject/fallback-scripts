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

# TODO: validate output from all 3 commands using grep, grep, and stem?

# List of relay fingerprints to test with generate and lookup
# moria1, Serge, no caches extra info x 2, no dir port x 2, doesn't exist
TEST_RELAY_LIST="
9695DFC35FFEB861329B9F1AB04C46397020CE31
BA44A889E64B93FAA2B114E02C2A279A8555C533
025B66CEBC070FCB0519D206CF0CF4965C20C96E
0338F9F55111FE8E3570E7DE117EF3AF999CC1D7
5AFAC3D00E97D6733112CC9CA2A788691FA87125
5DB9AE27A44EB7B476CC04A66C67A71C97A001E6
AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
"

# We need each relay fingerprint as a separate argument
# shellcheck disable=SC2086
$PYTHON generateFallbackDirLine.py $TEST_RELAY_LIST
# shellcheck disable=SC2086
$PYTHON lookupFallbackDirContact.py $TEST_RELAY_LIST

# If we get the top 200 relays from Onionoo, we generate a list with about
# 10 fallbacks
export TOR_FB_ONIONOO_LIMIT=200
$PYTHON updateFallbackDirs.py
