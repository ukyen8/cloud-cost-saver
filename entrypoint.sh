#!/bin/sh
set -e

# Positional arguments from action.yml
TEMPLATE="$1"
ENVIRONMENT="$2"
SAMCONFIG="$3"
CONFIG="$4"

ARGS="--template \"$TEMPLATE\" --environment \"$ENVIRONMENT\" --config \"$CONFIG\""

if [ -n "$SAMCONFIG" ]; then
  ARGS="$ARGS --samconfig \"$SAMCONFIG\""
fi

# Execute the Rust binary with the constructed arguments
exec /action/target/release/ccs $ARGS
