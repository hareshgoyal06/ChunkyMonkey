"use client";

import React, { useState } from "react";
import { API_BASE } from "@/app/api/config";

interface QueryResult {
  chunk_id: number;
  text: string;
  metadata: any;
  score: number;
}

interface QueryPanelProps {
  collectionId: number;
  onResultClick?: (chunkId: number) => void;
}

export default function QueryPanel({
  collectionId,
  onResultClick,
}: QueryPanelProps) {
  const [query, setQuery] = useState("");
  const [topK, setTopK] = useState(5);
  const [results, setResults] = useState<QueryResult[]>([]);
  const [isSearching, setIsSearching] = useState(false);

  const handleSearch = async () => {
    if (!query.trim()) return;

    setIsSearching(true);

    try {
      const response = await fetch(`${API_BASE}/query`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          collection_id: collectionId,
          query: query.trim(),
          top_k: topK,
        }),
      });

      if (!response.ok) {
        throw new Error(`Search failed: ${response.statusText}`);
      }

      const searchResults = await response.json();
      setResults(searchResults);
    } catch (error) {
      console.error("Search error:", error);
      alert("Search failed. Please try again.");
    } finally {
      setIsSearching(false);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      handleSearch();
    }
  };

  return (
    <div className="w-full p-6 border border-gray-300 rounded-lg bg-white">
      <h3 className="text-lg font-semibold mb-4 text-gray-700">
        Search Documents
      </h3>

      <div className="space-y-4">
        {/* Search Input */}
        <div className="flex gap-2">
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder="Enter your search query..."
            className="flex-1 px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            disabled={isSearching}
          />
          <select
            value={topK}
            onChange={(e) => setTopK(Number(e.target.value))}
            className="px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            disabled={isSearching}
          >
            <option value={3}>Top 3</option>
            <option value={5}>Top 5</option>
            <option value={10}>Top 10</option>
            <option value={20}>Top 20</option>
          </select>
          <button
            onClick={handleSearch}
            disabled={isSearching || !query.trim()}
            className={`px-4 py-2 rounded-md transition-colors ${
              isSearching || !query.trim()
                ? "bg-gray-300 text-gray-500 cursor-not-allowed"
                : "bg-blue-600 text-white hover:bg-blue-700"
            }`}
          >
            {isSearching ? (
              <div className="flex items-center gap-2">
                <div className="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                Search
              </div>
            ) : (
              "Search"
            )}
          </button>
        </div>

        {/* Results */}
        {results.length > 0 && (
          <div className="mt-6">
            <h4 className="text-md font-medium mb-3 text-gray-700">
              Results ({results.length})
            </h4>
            <div className="space-y-3 max-h-96 overflow-y-auto">
              {results.map((result, index) => (
                <div
                  key={result.chunk_id}
                  className="p-4 border border-gray-200 rounded-lg hover:bg-gray-50 cursor-pointer transition-colors"
                  onClick={() => onResultClick?.(result.chunk_id)}
                >
                  <div className="flex justify-between items-start mb-2">
                    <span className="text-sm font-medium text-gray-600">
                      Result #{index + 1}
                    </span>
                    <span className="text-sm text-green-600 font-medium">
                      {(result.score * 100).toFixed(1)}% match
                    </span>
                  </div>
                  <p className="text-gray-800 text-sm leading-relaxed">
                    {result.text.length > 200
                      ? `${result.text.substring(0, 200)}...`
                      : result.text}
                  </p>
                  {result.metadata &&
                    Object.keys(result.metadata).length > 0 && (
                      <div className="mt-2 text-xs text-gray-500">
                        <span className="font-medium">Metadata:</span>{" "}
                        {JSON.stringify(result.metadata)}
                      </div>
                    )}
                </div>
              ))}
            </div>
          </div>
        )}

        {results.length === 0 && !isSearching && query && (
          <div className="text-center py-8 text-gray-500">
            <p>No results found for "{query}"</p>
            <p className="text-sm mt-1">
              Try different keywords or check your collection
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
