"use client";

import React, { useEffect, useState } from "react";
import Plot from "react-plotly.js";
import { API_BASE } from "@/app/api/config";

interface ProjectionPoint {
  id: number;
  x: number;
  y: number;
  text: string;
}

interface ProjectionProps {
  collectionId: number;
  highlightedIds?: number[];
  onPointClick?: (chunkId: number) => void;
}

export default function Projection({
  collectionId,
  highlightedIds = [],
  onPointClick,
}: ProjectionProps) {
  const [points, setPoints] = useState<ProjectionPoint[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchProjection = async () => {
    setIsLoading(true);
    setError(null);

    try {
      const response = await fetch(`${API_BASE}/projection/${collectionId}`);

      if (!response.ok) {
        throw new Error(`Failed to fetch projection: ${response.statusText}`);
      }

      const data = await response.json();
      setPoints(data.points || []);
    } catch (err) {
      console.error("Projection fetch error:", err);
      setError(
        err instanceof Error ? err.message : "Failed to load projection"
      );
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    if (collectionId) {
      fetchProjection();
    }
  }, [collectionId]);

  const handlePointClick = (data: any) => {
    if (
      data.points &&
      data.points[0] &&
      data.points[0].pointIndex !== undefined
    ) {
      const pointIndex = data.points[0].pointIndex;
      const chunkId = points[pointIndex]?.id;
      if (chunkId && onPointClick) {
        onPointClick(chunkId);
      }
    }
  };

  if (isLoading) {
    return (
      <div className="w-full h-96 border border-gray-300 rounded-lg bg-white flex items-center justify-center">
        <div className="text-center">
          <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mb-4"></div>
          <p className="text-gray-600">Loading projection...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="w-full h-96 border border-gray-300 rounded-lg bg-white flex items-center justify-center">
        <div className="text-center">
          <p className="text-red-600 mb-4">{error}</p>
          <button
            onClick={fetchProjection}
            className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
          >
            Retry
          </button>
        </div>
      </div>
    );
  }

  if (points.length === 0) {
    return (
      <div className="w-full h-96 border border-gray-300 rounded-lg bg-white flex items-center justify-center">
        <div className="text-center text-gray-500">
          <p className="text-lg mb-2">No data to visualize</p>
          <p className="text-sm">Upload a document to see the 2D projection</p>
        </div>
      </div>
    );
  }

  // Prepare data for Plotly
  const x = points.map((p) => p.x);
  const y = points.map((p) => p.y);
  const texts = points.map((p) => p.text);
  const ids = points.map((p) => p.id);

  // Create marker colors based on highlighted IDs
  const colors = points.map((p) =>
    highlightedIds.includes(p.id) ? "#ff6b6b" : "#4ecdc4"
  );

  // Create marker sizes based on highlighted IDs
  const sizes = points.map((p) => (highlightedIds.includes(p.id) ? 12 : 8));

  return (
    <div className="w-full h-96 border border-gray-300 rounded-lg bg-white p-4">
      <div className="flex justify-between items-center mb-4">
        <h3 className="text-lg font-semibold text-gray-700">
          2D Projection ({points.length} points)
        </h3>
        <button
          onClick={fetchProjection}
          className="px-3 py-1 text-sm bg-gray-100 text-gray-700 rounded-md hover:bg-gray-200 transition-colors"
        >
          Refresh
        </button>
      </div>

      <Plot
        data={[
          {
            x: x,
            y: y,
            mode: "markers",
            type: "scatter",
            marker: {
              color: colors,
              size: sizes,
              opacity: 0.7,
            },
            text: texts,
            hovertemplate: "<b>Chunk %{text}</b><br><br>%{text}<extra></extra>",
            hoverinfo: "text",
            ids: ids,
          },
        ]}
        layout={{
          width: undefined,
          height: 300,
          margin: { l: 50, r: 50, t: 50, b: 50 },
          xaxis: { title: "UMAP 1" },
          yaxis: { title: "UMAP 2" },
          hovermode: "closest",
          showlegend: false,
        }}
        config={{
          displayModeBar: false,
          responsive: true,
        }}
        onClick={handlePointClick}
        style={{ width: "100%", height: "100%" }}
      />

      {highlightedIds.length > 0 && (
        <div className="mt-4 text-sm text-gray-600">
          <span className="font-medium">Highlighted:</span>{" "}
          {highlightedIds.length} search result(s)
        </div>
      )}
    </div>
  );
}
