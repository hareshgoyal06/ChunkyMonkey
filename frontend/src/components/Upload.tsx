"use client";

import React, { useState } from "react";
import { API_BASE } from "@/app/api/config";

interface UploadProps {
  collectionId: number;
  onDone: (result: { chunks: number }) => void;
}

export default function Upload({ collectionId, onDone }: UploadProps) {
  const [isUploading, setIsUploading] = useState(false);
  const [dragActive, setDragActive] = useState(false);

  const handleFileUpload = async (file: File) => {
    if (!file.name.endsWith(".txt")) {
      alert("Please upload a .txt file");
      return;
    }

    setIsUploading(true);

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
      onDone(result);
    } catch (error) {
      console.error("Upload error:", error);
      alert("Upload failed. Please try again.");
    } finally {
      setIsUploading(false);
    }
  };

  const handleDrag = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.type === "dragenter" || e.type === "dragover") {
      setDragActive(true);
    } else if (e.type === "dragleave") {
      setDragActive(false);
    }
  };

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragActive(false);

    if (e.dataTransfer.files && e.dataTransfer.files[0]) {
      handleFileUpload(e.dataTransfer.files[0]);
    }
  };

  const handleFileInput = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      handleFileUpload(e.target.files[0]);
    }
  };

  return (
    <div className="w-full p-6 border border-gray-300 rounded-lg bg-white">
      <div className="text-center mb-6">
        <h3 className="text-xl font-semibold mb-2 text-gray-700">
          Add Documents to Vector Database
        </h3>
        <p className="text-gray-600">
          Upload text documents to see how they're chunked, embedded, and made
          searchable
        </p>
      </div>

      <div
        className={`border-2 border-dashed rounded-lg p-8 text-center transition-colors ${
          dragActive
            ? "border-blue-500 bg-blue-50"
            : "border-gray-300 hover:border-gray-400"
        }`}
        onDragEnter={handleDrag}
        onDragLeave={handleDrag}
        onDragOver={handleDrag}
        onDrop={handleDrop}
      >
        <div className="space-y-4">
          <div className="text-6xl text-gray-400">📄</div>
          <div>
            <p className="text-lg text-gray-600 mb-2">
              {isUploading
                ? "Processing document..."
                : "Drag and drop a .txt file here"}
            </p>
            <p className="text-sm text-gray-500 mb-4">or click to browse</p>
            <input
              type="file"
              accept=".txt"
              onChange={handleFileInput}
              disabled={isUploading}
              className="hidden"
              id="file-upload"
            />
            <label
              htmlFor="file-upload"
              className={`inline-block px-4 py-2 rounded-md cursor-pointer transition-colors ${
                isUploading
                  ? "bg-gray-300 text-gray-500 cursor-not-allowed"
                  : "bg-blue-600 text-white hover:bg-blue-700"
              }`}
            >
              {isUploading ? "Processing..." : "Choose File"}
            </label>
          </div>
        </div>
      </div>

      {isUploading && (
        <div className="mt-4 text-center">
          <div className="inline-block animate-spin rounded-full h-6 w-6 border-b-2 border-blue-600"></div>
          <p className="mt-2 text-sm text-gray-600">
            Chunking document and creating embeddings...
          </p>
        </div>
      )}

      {/* Educational Info */}
      <div className="mt-6 p-4 bg-gray-50 rounded-lg">
        <h4 className="font-medium text-gray-800 mb-2">
          What happens when you upload:
        </h4>
        <ul className="text-sm text-gray-600 space-y-1">
          <li>
            • Document is split into 800-character chunks with 150-character
            overlap
          </li>
          <li>
            • Each chunk is converted to a 384-dimensional vector embedding
          </li>
          <li>
            • Vectors are stored in PostgreSQL with pgvector for fast similarity
            search
          </li>
          <li>
            • 2D visualization shows the semantic relationships between chunks
          </li>
        </ul>
      </div>
    </div>
  );
}
