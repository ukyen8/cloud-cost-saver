#!/bin/sh
set -e

# Start building the command
CMD="/usr/src/app/target/release/ccs scan"

# Required arguments
if [ -n "$INPUT_TEMPLATE" ]; then
  CMD="$CMD --template $INPUT_TEMPLATE"
fi

if [ -n "$INPUT_CONFIG" ]; then
  CMD="$CMD --config $INPUT_CONFIG"
fi

# Optional arguments
if [ -n "$INPUT_ENVIRONMENT" ]; then
  CMD="$CMD --environment $INPUT_ENVIRONMENT"
fi

if [ -n "$INPUT_SAMCONFIG" ]; then
  CMD="$CMD --samconfig $INPUT_SAMCONFIG"
fi

if [ -n "$INPUT_PRESET" ]; then
  CMD="$CMD --preset $INPUT_PRESET"
fi

if [ -n "$INPUT_FORMAT" ]; then
  CMD="$CMD --format $INPUT_FORMAT"
fi

# Execute the constructed command
echo "Running: $CMD"
exec $CMD
