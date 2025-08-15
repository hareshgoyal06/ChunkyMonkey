#!/bin/bash

echo "🚀 Starting Drag-n-Vector..."

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

# Start the database
echo "📦 Starting PostgreSQL with pgvector..."
docker compose up -d

# Wait for database to be ready
echo "⏳ Waiting for database to be ready..."
sleep 10

# Start backend
echo "🐍 Starting FastAPI backend..."
cd backend
if [ ! -d ".venv" ]; then
    echo "📦 Creating Python virtual environment..."
    python -m venv .venv
fi

source .venv/bin/activate
pip install -r requirements.txt > /dev/null 2>&1

echo "🌐 Backend starting on http://localhost:8000"
uvicorn app:app --reload --port 8000 &
BACKEND_PID=$!

# Wait for backend to start
sleep 5

# Start frontend
echo "⚛️  Starting Next.js frontend..."
cd ../frontend
npm install > /dev/null 2>&1

echo "🌐 Frontend starting on http://localhost:3000"
npm run dev &
FRONTEND_PID=$!

echo ""
echo "✅ Drag-n-Vector is starting up!"
echo "📊 Backend: http://localhost:8000"
echo "🎨 Frontend: http://localhost:3000"
echo ""
echo "Press Ctrl+C to stop all services"

# Wait for user to stop
trap "echo ''; echo '🛑 Stopping services...'; kill $BACKEND_PID $FRONTEND_PID; docker compose down; echo '✅ All services stopped'; exit" INT

wait 