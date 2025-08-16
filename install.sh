#!/bin/bash

echo "🐒 Installing ChunkyMonkey - Going Bananas for Chunks! 🍌"
echo "=========================================================="

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Build the project
echo "📦 Building ChunkyMonkey..."
cargo build --release

if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    
    # Create symlink for easy access
    if [ ! -L /usr/local/bin/chunkymonkey ]; then
        echo "🔗 Creating symlink..."
        sudo ln -sf "$(pwd)/target/release/chunkymonkey" /usr/local/bin/chunkymonkey
        echo "✅ ChunkyMonkey installed to /usr/local/bin/chunkymonkey"
    else
        echo "✅ ChunkyMonkey already installed"
    fi
    
    # Create 'cm' alias
    if [ ! -L /usr/local/bin/cm ]; then
        echo "🔗 Creating 'cm' alias..."
        sudo ln -sf "$(pwd)/target/release/chunkymonkey" /usr/local/bin/cm
        echo "✅ 'cm' alias created at /usr/local/bin/cm"
    else
        echo "✅ 'cm' alias already exists"
    fi
    
    echo ""
    echo "🎉 Installation complete!"
    echo ""
    echo "Usage:"
    echo "  chunkymonkey start          # Full command (recommended)"
    echo "  chunkymonkey interactive    # Full command (alternative)"
    echo "  cm start                    # Short alias (recommended)"
    echo "  cm interactive              # Short alias (alternative)"
    echo ""
    echo "  chunkymonkey index /path    # Full command"
    echo "  cm index /path              # Short alias"
    echo ""
    echo "  chunkymonkey search \"query\" # Full command"
    echo "  cm search \"query\"           # Short alias"
    echo ""
    echo "For help: chunkymonkey --help or cm --help"
    echo ""
    echo "🐒 Ready to go bananas for chunks! 🍌"
else
    echo "❌ Build failed!"
    exit 1
fi 