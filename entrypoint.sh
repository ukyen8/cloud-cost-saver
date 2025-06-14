#!/bin/sh
set -e

# Positional arguments from action.yml
TEMPLATE="$1"
ENVIRONMENT="$2"
SAMCONFIG="$3"
CONFIG="$4"
CLOUD_PROVIDER="$5"

ARGS="--template \"$TEMPLATE\" --environment \"$ENVIRONMENT\" --config \"$CONFIG\""

if [ -n "$SAMCONFIG" ]; then
  ARGS="$ARGS --samconfig \"$SAMCONFIG\""
fi

echo "Listing files in /action:"
ls -l /action/src/fixtures

echo "Listing files in /action/src/fixtures/aws:"
ls -l /action/src/fixtures/aws

# Execute the Rust binary with the constructed arguments and cloud provider
echo "Running: /action/target/release/ccs $ARGS \"$CLOUD_PROVIDER\""
exec /action/target/release/ccs $ARGS "$CLOUD_PROVIDER"
