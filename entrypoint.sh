#!/bin/sh
set -e

# Positional arguments from action.yml
TEMPLATE="$1"
ENVIRONMENT="$2"
SAMCONFIG="$3"
CONFIG="$4"
CLOUD_PROVIDER="$5"

# Build ARGS without extra quotes
ARGS="--template $TEMPLATE --environment $ENVIRONMENT --config $CONFIG"

if [ -n "$SAMCONFIG" ]; then
  ARGS="$ARGS --samconfig $SAMCONFIG"
fi


# Execute the Rust binary with the constructed arguments and cloud provider
echo "Running: /target/release/ccs $ARGS \"$CLOUD_PROVIDER\""
exec /target/release/ccs $ARGS "$CLOUD_PROVIDER"
