#!/bin/sh
set -e

# Positional arguments from action.yml
TEMPLATE="/action/$1"
ENVIRONMENT="$2"
SAMCONFIG="/action/$3"
CONFIG="/action/$4"
CLOUD_PROVIDER="$5"

# Build ARGS without extra quotes
ARGS="--template $TEMPLATE --environment $ENVIRONMENT --config $CONFIG"

if [ -n "$SAMCONFIG" ]; then
  ARGS="$ARGS --samconfig $SAMCONFIG"
fi


# Execute the Rust binary with the constructed arguments and cloud provider
echo "Running: /action/target/release/ccs $ARGS \"$CLOUD_PROVIDER\""
exec /action/target/release/ccs $ARGS "$CLOUD_PROVIDER"
