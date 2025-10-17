#!/bin/bash
# Copytrader Bot Startup Script
# This script runs the ingestion bot and automatically restarts it if it crashes

set -e

# Change to bot directory
cd "$(dirname "$0")"

echo "=========================================="
echo "Copytrader Bot - Starting..."
echo "Time: $(date)"
echo "=========================================="

# Build in release mode for better performance
echo "Building bot in release mode..."
cargo build --release -p grpc_subscriber

# Run the bot with automatic restart on crash
while true; do
    echo ""
    echo "Starting bot at $(date)"
    echo "------------------------------------------"
    
    # Run the bot - will restart if it exits
    cargo run --release -p grpc_subscriber
    
    EXIT_CODE=$?
    echo ""
    echo "Bot exited with code $EXIT_CODE at $(date)"
    
    if [ $EXIT_CODE -eq 0 ]; then
        echo "Bot exited cleanly. Waiting 5 seconds before restart..."
    else
        echo "Bot crashed! Waiting 10 seconds before restart..."
        sleep 10
    fi
    
    sleep 5
done
