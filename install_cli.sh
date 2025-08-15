#!/bin/bash

echo "🚀 Installing CLI Vector Search Tool..."

# Check if Python 3 is available
if ! command -v python3 &> /dev/null; then
    echo "❌ Python 3 is required but not installed"
    exit 1
fi

# Create virtual environment if it doesn't exist
if [ ! -d ".venv" ]; then
    echo "📦 Creating virtual environment..."
    python3 -m venv .venv
fi

# Activate virtual environment
echo "🔧 Activating virtual environment..."
source .venv/bin/activate

# Install dependencies
echo "📥 Installing dependencies..."
pip install -r cli_requirements.txt

# Make CLI executable
chmod +x cli_vector_search.py

# Create symlink for easy access
if [ ! -L /usr/local/bin/vsearch ]; then
    echo "🔗 Creating symlink..."
    sudo ln -sf "$(pwd)/cli_vector_search.py" /usr/local/bin/vsearch
fi

echo "✅ Installation complete!"
echo ""
echo "Usage examples:"
echo "  vsearch index /path/to/your/codebase"
echo "  vsearch search 'function definition'"
echo "  vsearch ask 'How does authentication work?'"
echo "  vsearch interactive"
echo "  vsearch stats"
echo ""
echo "For help: vsearch --help" 