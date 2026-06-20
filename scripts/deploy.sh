#!/bin/bash

# Determine the script directory and project root
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"

# Change to project root to ensure artifacts are accessible
cd "$PROJECT_ROOT"

echo "🚀 Starting deployment process..."
echo "📂 Working directory: $PROJECT_ROOT"
echo ""

# Run store script
echo "📦 Step 1: Storing contracts..."
bash "$SCRIPT_DIR/store.sh"
if [ $? -ne 0 ]; then
    echo "❌ Store script failed!"
    exit 1
fi

echo ""
echo "✅ Contracts stored successfully!"
echo ""

# Run init script
echo "📦 Step 2: Initializing controller..."
bash "$SCRIPT_DIR/init.sh"
if [ $? -ne 0 ]; then
    echo "❌ Init script failed!"
    exit 1
fi

echo ""
echo "🎉 Deployment completed successfully!"