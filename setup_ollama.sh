#!/bin/bash

echo "🐒 Setting up Ollama for ChunkyMonkey..."

# Check if Ollama is already installed
if command -v ollama &> /dev/null; then
    echo "✅ Ollama is already installed"
else
    echo "📥 Installing Ollama..."
    curl -fsSL https://ollama.ai/install.sh | sh
fi

# Start Ollama service
echo "🔧 Starting Ollama service..."
ollama serve &
OLLAMA_PID=$!

# Wait for service to start
echo "⏳ Waiting for Ollama service to start..."
sleep 5

# Pull a model for embeddings
echo "📦 Pulling Llama2 model for embeddings..."
ollama pull llama2:13b

# Stop the service
kill $OLLAMA_PID

echo ""
echo "🎉 Ollama setup complete!"
echo ""
echo "To use Ollama with ChunkyMonkey:"
echo "1. Start Ollama: ollama serve"
echo "2. Set your .env file with:"
echo "   OPENAI_API_KEY="
echo "   OLLAMA_BASE_URL=http://localhost:11434"
echo "   OLLAMA_MODEL=llama2:13b"
echo ""
echo "3. Run ChunkyMonkey: cargo run -- start"
echo ""
echo "🐒 Ready to go bananas for chunks! 🍌" 