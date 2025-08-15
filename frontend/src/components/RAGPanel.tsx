"use client";

import React, { useState } from "react";
import { API_BASE } from "@/app/api/config";

interface RAGSource {
  chunk_id: number;
  text: string;
  score: number;
}

interface RAGResponse {
  query: string;
  context: string[];
  answer: string;
  sources: RAGSource[];
}

interface RAGPanelProps {
  collectionId: number;
}

export default function RAGPanel({ collectionId }: RAGPanelProps) {
  const [query, setQuery] = useState("");
  const [topK, setTopK] = useState(5);
  const [includeContext, setIncludeContext] = useState(true);
  const [result, setResult] = useState<RAGResponse | null>(null);
  const [isProcessing, setIsProcessing] = useState(false);

  const handleRAGQuery = async () => {
    if (!query.trim()) return;

    setIsProcessing(true);

    try {
      const response = await fetch(`${API_BASE}/rag`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          collection_id: collectionId,
          query: query.trim(),
          top_k: topK,
          include_context: includeContext,
        }),
      });

      if (!response.ok) {
        throw new Error(`RAG query failed: ${response.statusText}`);
      }

      const ragResult = await response.json();
      setResult(ragResult);
    } catch (error) {
      console.error("RAG error:", error);
      alert("RAG query failed. Please try again.");
    } finally {
      setIsProcessing(false);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      handleRAGQuery();
    }
  };

  return (
    <div className="w-full p-6 border border-gray-300 rounded-lg bg-white">
      <div className="text-center mb-6">
        <h3 className="text-xl font-semibold mb-2 text-gray-700">
          RAG Pipeline Demo
        </h3>
        <p className="text-gray-600">
          Retrieval-Augmented Generation: Ask questions and get AI-generated
          answers based on your documents
        </p>
      </div>

      <div className="space-y-4">
        {/* Query Input */}
        <div className="space-y-3">
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder="Ask a question about your documents..."
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            disabled={isProcessing}
          />

          <div className="flex gap-4 items-center">
            <select
              value={topK}
              onChange={(e) => setTopK(Number(e.target.value))}
              className="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              disabled={isProcessing}
            >
              <option value={3}>Top 3 chunks</option>
              <option value={5}>Top 5 chunks</option>
              <option value={10}>Top 10 chunks</option>
            </select>

            <label className="flex items-center space-x-2">
              <input
                type="checkbox"
                checked={includeContext}
                onChange={(e) => setIncludeContext(e.target.checked)}
                className="rounded"
                disabled={isProcessing}
              />
              <span className="text-sm text-gray-600">
                Include context in answer
              </span>
            </label>
          </div>

          <button
            onClick={handleRAGQuery}
            disabled={isProcessing || !query.trim()}
            className={`w-full px-4 py-2 rounded-md transition-colors ${
              isProcessing || !query.trim()
                ? "bg-gray-300 text-gray-500 cursor-not-allowed"
                : "bg-purple-600 text-white hover:bg-purple-700"
            }`}
          >
            {isProcessing ? (
              <div className="flex items-center justify-center gap-2">
                <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                Generating Answer...
              </div>
            ) : (
              "Generate Answer"
            )}
          </button>
        </div>

        {/* Example Questions */}
        <div className="p-3 bg-purple-50 rounded-lg">
          <p className="text-sm font-medium text-purple-800 mb-2">
            Try these example questions:
          </p>
          <div className="flex flex-wrap gap-2">
            {[
              "What is machine learning?",
              "How does deep learning work?",
              "What are the types of machine learning?",
              "Explain data preprocessing",
            ].map((example) => (
              <button
                key={example}
                onClick={() => setQuery(example)}
                className="px-2 py-1 text-xs bg-purple-100 text-purple-700 rounded hover:bg-purple-200 transition-colors"
              >
                {example}
              </button>
            ))}
          </div>
        </div>

        {/* Results */}
        {result && (
          <div className="mt-6 space-y-4">
            <div className="p-4 bg-gray-50 rounded-lg">
              <h4 className="font-medium text-gray-800 mb-2">
                Generated Answer:
              </h4>
              <div className="text-sm text-gray-700 whitespace-pre-wrap">
                {result.answer}
              </div>
            </div>

            <div className="p-4 bg-blue-50 rounded-lg">
              <h4 className="font-medium text-blue-800 mb-2">
                Sources ({result.sources.length} chunks):
              </h4>
              <div className="space-y-2 max-h-48 overflow-y-auto">
                {result.sources.map((source, index) => (
                  <div
                    key={source.chunk_id}
                    className="p-3 bg-white rounded border border-blue-200"
                  >
                    <div className="flex justify-between items-start mb-1">
                      <span className="text-xs font-medium text-blue-600">
                        Source #{index + 1}
                      </span>
                      <span className="text-xs text-green-600">
                        {(source.score * 100).toFixed(1)}% relevant
                      </span>
                    </div>
                    <p className="text-xs text-gray-700">{source.text}</p>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}
      </div>

      {/* How RAG Works */}
      <div className="mt-6 p-4 bg-gray-50 rounded-lg">
        <h4 className="font-medium text-gray-800 mb-2">How RAG works:</h4>
        <ul className="text-sm text-gray-600 space-y-1">
          <li>• Your question is converted to a vector embedding</li>
          <li>• Database retrieves the most relevant document chunks</li>
          <li>• AI generates an answer based on the retrieved context</li>
          <li>• Sources are provided for transparency and verification</li>
        </ul>
      </div>
    </div>
  );
}
