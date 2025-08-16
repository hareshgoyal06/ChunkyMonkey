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
    echo "Usage examples:"
    echo "  cm start                    # Start ChunkyMonkey (recommended)"
    echo "  cm index /path              # Index a directory"
    echo "  cm search query             # Search for content"
    echo "  cm ask question             # Ask a question using RAG"
    echo "  cm stats                    # Show database statistics"
    echo "  cm rag-stats                # Show RAG pipeline statistics"
    echo "  cm clear                    # Clear all indexed data"
    echo ""
    echo "🐒 Ready to go bananas for chunks! 🍌"
else
    echo "❌ Failed to create 'cm' alias"
    exit 1
fi
