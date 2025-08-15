"use client";

import React, { useState, useEffect, useRef } from "react";
import Canvas from "@/components/Canvas";
import Upload from "@/components/Upload";
import QueryPanel from "@/components/QueryPanel";
import Projection from "@/components/Projection";
import { API_BASE } from "@/app/api/config";

export default function Home() {
  const [collectionId, setCollectionId] = useState<number | null>(null);
  const [highlightedIds, setHighlightedIds] = useState<number[]>([]);
  const [showUpload, setShowUpload] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Initialize collection on mount
  useEffect(() => {
    const initCollection = async () => {
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
      } catch (error) {
        console.error("Error initializing collection:", error);
      }
    };

    initCollection();
  }, []);

  const handleRunPipeline = () => {
    if (fileInputRef.current) {
      fileInputRef.current.click();
    }
  };

  const handleFileUpload = async (
    event: React.ChangeEvent<HTMLInputElement>
  ) => {
    const file = event.target.files?.[0];
    if (!file || !collectionId) return;

    if (!file.name.endsWith(".txt")) {
      alert("Please upload a .txt file");
      return;
    }

    try {
      const formData = new FormData();
      formData.append("collection_id", collectionId.toString());
      formData.append("file", file);
      formData.append("chunk_size", "800");
      formData.append("overlap", "150");

      const response = await fetch(`${API_BASE}/ingest/file`, {
        method: "POST",
        body: formData,
      });

      if (!response.ok) {
        throw new Error(`Upload failed: ${response.statusText}`);
      }

      const result = await response.json();
      alert(`Successfully processed ${result.chunks} chunks!`);

      // Refresh projection
      window.location.reload();
    } catch (error) {
      console.error("Upload error:", error);
      alert("Upload failed. Please try again.");
    } finally {
      // Reset file input
      if (fileInputRef.current) {
        fileInputRef.current.value = "";
      }
    }
  };

  const handleUploadDone = (result: { chunks: number }) => {
    alert(`Successfully processed ${result.chunks} chunks!`);
    setShowUpload(false);
  };

  const handleResultClick = (chunkId: number) => {
    setHighlightedIds([chunkId]);
  };

  if (!collectionId) {
    return (
      <div className="min-h-screen bg-gray-50 flex items-center justify-center">
        <div className="text-center">
          <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mb-4"></div>
          <p className="text-gray-600">Initializing application...</p>
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
                Drag-n-Vector
              </h1>
              <span className="ml-3 px-2 py-1 text-xs bg-blue-100 text-blue-800 rounded-full">
                Collection #{collectionId}
              </span>
            </div>
            <div className="flex items-center space-x-4">
              <button
                onClick={() => setShowUpload(!showUpload)}
                className="px-4 py-2 text-sm bg-gray-100 text-gray-700 rounded-md hover:bg-gray-200 transition-colors"
              >
                {showUpload ? "Hide Upload" : "Show Upload"}
              </button>
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="space-y-8">
          {/* Pipeline Canvas */}
          <Canvas onRun={handleRunPipeline} />

          {/* Hidden file input for Run button */}
          <input
            ref={fileInputRef}
            type="file"
            accept=".txt"
            onChange={handleFileUpload}
            className="hidden"
          />

          {/* Upload Section (conditionally shown) */}
          {showUpload && (
            <Upload collectionId={collectionId} onDone={handleUploadDone} />
          )}

          {/* Search and Visualization */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
            {/* Query Panel */}
            <QueryPanel
              collectionId={collectionId}
              onResultClick={handleResultClick}
            />

            {/* Projection */}
            <Projection
              collectionId={collectionId}
              highlightedIds={highlightedIds}
              onPointClick={handleResultClick}
            />
          </div>
        </div>
      </main>

      {/* Footer */}
      <footer className="bg-white border-t border-gray-200 mt-16">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4">
          <p className="text-center text-sm text-gray-500">
            Drag-n-Vector - Vector Search with UMAP Visualization
          </p>
        </div>
      </footer>
    </div>
  );
}
