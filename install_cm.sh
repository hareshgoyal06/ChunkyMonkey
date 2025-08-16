#!/bin/bash

echo "🐒 Installing 'cm' alias for ChunkyMonkey..."

# Check if ChunkyMonkey binary exists
if [ ! -f "./target/release/chunkymonkey" ]; then
    echo "❌ ChunkyMonkey binary not found. Please run 'cargo build --release' first."
    exit 1
fi

# Create 'cm' alias in /usr/local/bin
echo "🔗 Creating 'cm' alias..."
sudo ln -sf "$(pwd)/target/release/chunkymonkey" /usr/local/bin/cm

if [ $? -eq 0 ]; then
    echo "✅ 'cm' alias installed successfully!"
    echo ""
    echo "You can now use:"
    echo "  cm --help"
    echo "  cm start                    # Start interactive mode (recommended)"
    echo "  cm interactive              # Start interactive mode (alternative)"
    echo "  cm index /path/to/documents"
    echo "  cm search \"your query\""
    echo ""
    echo "🐒 Ready to go bananas for chunks! 🍌"
else
    echo "❌ Failed to create 'cm' alias"
    exit 1
fi
