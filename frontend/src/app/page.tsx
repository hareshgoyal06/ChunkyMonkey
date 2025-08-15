"use client";

import React, { useState, useEffect, useRef } from "react";
import Upload from "@/components/Upload";
import QueryPanel from "@/components/QueryPanel";
import RAGPanel from "@/components/RAGPanel";
import Projection from "@/components/Projection";
import { API_BASE } from "@/app/api/config";

type TabType = "search" | "rag" | "visualization";

export default function Home() {
  const [collectionId, setCollectionId] = useState<number | null>(null);
  const [highlightedIds, setHighlightedIds] = useState<number[]>([]);
  const [showUpload, setShowUpload] = useState(true);
  const [activeTab, setActiveTab] = useState<TabType>("search");
  const [stats, setStats] = useState<{
    chunks: number;
    documents: number;
  } | null>(null);

  // Initialize collection on mount
  useEffect(() => {
    const initCollection = async () => {
      try {
        // Use collection ID 1 by default (where the data is)
        setCollectionId(1);

        // Optionally verify the collection exists
        const response = await fetch(`${API_BASE}/debug/1`);
        if (response.ok) {
          const debugData = await response.json();
          console.log("Collection 1 status:", debugData);
          setStats({
            chunks: debugData.chunks_count,
            documents: debugData.documents.length,
          });
        }
      } catch (error) {
        console.error("Error initializing collection:", error);
        // Fallback: create a new collection if needed
        try {
          const formData = new FormData();
          formData.append("name", "Demo");

          const response = await fetch(`${API_BASE}/collections/create`, {
            method: "POST",
            body: formData,
          });

          if (response.ok) {
            const result = await response.json();
            setCollectionId(result.id);
          } else {
            console.error("Failed to create collection");
          }
        } catch (createError) {
          console.error("Error creating collection:", createError);
        }
      }
    };

    initCollection();
  }, []);

  const handleUploadDone = (result: { chunks: number }) => {
    alert(`Successfully processed ${result.chunks} chunks!`);
    setShowUpload(false);
    // Refresh stats
    window.location.reload();
  };

  const handleResultClick = (chunkId: number) => {
    setHighlightedIds([chunkId]);
  };

  if (!collectionId) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-center">
          <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mb-4"></div>
          <p className="text-gray-600">Initializing vector database...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Header */}
      <header className="bg-white shadow-sm border-b border-gray-200">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center h-16">
            <div className="flex items-center">
              <h1 className="text-2xl font-bold text-gray-900">
                Vector Database Demo
              </h1>
              <span className="ml-3 px-2 py-1 text-xs bg-blue-100 text-blue-800 rounded-full">
                Collection #{collectionId}
              </span>
              {stats && (
                <span className="ml-2 text-sm text-gray-600">
                  {stats.chunks} chunks • {stats.documents} documents
                </span>
              )}
            </div>
            <div className="flex items-center space-x-4">
              <button
                onClick={() => setShowUpload(!showUpload)}
                className="px-4 py-2 text-sm bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
              >
                {showUpload ? "Hide Upload" : "Add Document"}
              </button>
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="space-y-8">
          {/* How it Works Section */}
          <div className="bg-white p-6 rounded-lg border border-gray-200">
            <h2 className="text-xl font-semibold mb-4 text-gray-800">
              How Vector Search Works
            </h2>
            <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
              <div className="text-center">
                <div className="w-12 h-12 bg-blue-100 rounded-full flex items-center justify-center mx-auto mb-3">
                  <span className="text-blue-600 font-bold">1</span>
                </div>
                <h3 className="font-medium text-gray-800 mb-2">
                  Document Chunking
                </h3>
                <p className="text-sm text-gray-600">
                  Large documents are split into smaller chunks (800 characters)
                  with overlap to maintain context.
                </p>
              </div>
              <div className="text-center">
                <div className="w-12 h-12 bg-green-100 rounded-full flex items-center justify-center mx-auto mb-3">
                  <span className="text-green-600 font-bold">2</span>
                </div>
                <h3 className="font-medium text-gray-800 mb-2">
                  Vector Embeddings
                </h3>
                <p className="text-sm text-gray-600">
                  Each chunk is converted to a 384-dimensional vector using
                  sentence transformers.
                </p>
              </div>
              <div className="text-center">
                <div className="w-12 h-12 bg-purple-100 rounded-full flex items-center justify-center mx-auto mb-3">
                  <span className="text-purple-600 font-bold">3</span>
                </div>
                <h3 className="font-medium text-gray-800 mb-2">
                  Semantic Search
                </h3>
                <p className="text-sm text-gray-600">
                  Queries are embedded and matched using cosine similarity to
                  find the most relevant chunks.
                </p>
              </div>
              <div className="text-center">
                <div className="w-12 h-12 bg-orange-100 rounded-full flex items-center justify-center mx-auto mb-3">
                  <span className="text-orange-600 font-bold">4</span>
                </div>
                <h3 className="font-medium text-gray-800 mb-2">
                  RAG Generation
                </h3>
                <p className="text-sm text-gray-600">
                  Retrieved chunks are used to generate contextual answers to
                  questions.
                </p>
              </div>
            </div>
          </div>

          {/* Upload Section */}
          {showUpload && (
            <Upload collectionId={collectionId} onDone={handleUploadDone} />
          )}

          {/* Tabs */}
          <div className="bg-white rounded-lg border border-gray-200">
            <div className="border-b border-gray-200">
              <nav className="flex space-x-8 px-6" aria-label="Tabs">
                <button
                  onClick={() => setActiveTab("search")}
                  className={`py-4 px-1 border-b-2 font-medium text-sm ${
                    activeTab === "search"
                      ? "border-blue-500 text-blue-600"
                      : "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                  }`}
                >
                  Semantic Search
                </button>
                <button
                  onClick={() => setActiveTab("rag")}
                  className={`py-4 px-1 border-b-2 font-medium text-sm ${
                    activeTab === "rag"
                      ? "border-purple-500 text-purple-600"
                      : "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                  }`}
                >
                  RAG Pipeline
                </button>
                <button
                  onClick={() => setActiveTab("visualization")}
                  className={`py-4 px-1 border-b-2 font-medium text-sm ${
                    activeTab === "visualization"
                      ? "border-green-500 text-green-600"
                      : "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                  }`}
                >
                  3D Visualization
                </button>
              </nav>
            </div>

            <div className="p-6">
              {activeTab === "search" && (
                <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
                  <QueryPanel
                    collectionId={collectionId}
                    onResultClick={handleResultClick}
                  />
                  <Projection
                    collectionId={collectionId}
                    highlightedIds={highlightedIds}
                    onPointClick={handleResultClick}
                  />
                </div>
              )}

              {activeTab === "rag" && (
                <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
                  <RAGPanel collectionId={collectionId} />
                  <Projection
                    collectionId={collectionId}
                    highlightedIds={highlightedIds}
                    onPointClick={handleResultClick}
                  />
                </div>
              )}

              {activeTab === "visualization" && (
                <div className="w-full">
                  <Projection
                    collectionId={collectionId}
                    highlightedIds={highlightedIds}
                    onPointClick={handleResultClick}
                  />
                </div>
              )}
            </div>
          </div>
        </div>
      </main>

      {/* Footer */}
      <footer className="bg-white border-t border-gray-200 mt-16">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
          <p className="text-center text-sm text-gray-500">
            Vector Database Demo - Powered by PostgreSQL + pgvector + UMAP + RAG
          </p>
        </div>
      </footer>
    </div>
  );
}
