#!/bin/sh
set -e

# Positional arguments from action.yml
TEMPLATE="/action/$1"
ENVIRONMENT="/action/$2"
SAMCONFIG="/action/$3"
CONFIG="/action/$4"
CLOUD_PROVIDER="$5"

# Build ARGS without extra quotes
ARGS="--template $TEMPLATE --environment $ENVIRONMENT --config $CONFIG"

if [ -n "$SAMCONFIG" ]; then
  ARGS="$ARGS --samconfig $SAMCONFIG"
fi

echo "Listing files in /action/src/fixtures:"
ls -l /action/src/fixtures

echo "Listing files in /action/src/fixtures/aws:"
ls -l /action/src/fixtures/aws
# Check if all argument files exist
for f in "$TEMPLATE" "$SAMCONFIG" "$CONFIG"; do
  if [ -n "$f" ] && [ ! -f "$f" ]; then
    echo "ERROR: File not found: $f" >&2
    exit 1
  fi
done
echo "Contents of config file ($CONFIG):"
cat "$CONFIG" || echo "Could not read $CONFIG"
# Execute the Rust binary with the constructed arguments and cloud provider
echo "Running: /action/target/release/ccs $ARGS \"$CLOUD_PROVIDER\""
exec /action/target/release/ccs $ARGS "$CLOUD_PROVIDER"
