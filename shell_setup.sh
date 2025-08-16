#!/bin/bash

# ChunkyMonkey Shell Setup 🐒🍌
# Source this file in your shell profile to add 'cm' to your PATH

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Add the current directory to PATH for the 'cm' script
export PATH="$SCRIPT_DIR:$PATH"

# Create an alias for even faster access (optional)
alias cm="$SCRIPT_DIR/target/release/chunkymonkey"

echo "🐒 ChunkyMonkey shell setup complete!"
echo "You can now use 'cm' from anywhere in your shell."
echo "To make this permanent, add this line to your ~/.bashrc or ~/.zshrc:"
echo "source $SCRIPT_DIR/shell_setup.sh"
