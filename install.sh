#!/bin/bash

echo "🔍 Installing TLDR - Too Long; Didn't Read"
echo "=========================================="

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Build the project
echo "📦 Building TLDR..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    
    # Create symlink for easy access
    if [ ! -L /usr/local/bin/tldr ]; then
        echo "🔗 Creating symlink..."
        sudo ln -sf "$(pwd)/target/release/tldr" /usr/local/bin/tldr
        echo "✅ TLDR installed to /usr/local/bin/tldr"
    else
        echo "✅ TLDR already installed"
    fi
    
    echo ""
    echo "🎉 Installation complete!"
    echo ""
    echo "Usage:"
    echo "  tldr index /path/to/your/project"
    echo "  tldr search \"authentication function\""
    echo "  tldr ask \"How does the API work?\""
    echo "  tldr interactive"
    echo ""
    echo "For help: tldr --help"
else
    echo "❌ Build failed!"
    exit 1
fi 