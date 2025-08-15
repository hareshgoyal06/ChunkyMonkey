"use client";

import React, { useCallback, useRef } from "react";
import ReactFlow, {
  Node,
  Edge,
  addEdge,
  Connection,
  useNodesState,
  useEdgesState,
  MiniMap,
  Controls,
  Background,
  NodeTypes,
} from "react-flow-renderer";
import "react-flow-renderer/dist/style.css";

interface CanvasProps {
  onRun: () => void;
}

const initialNodes: Node[] = [
  {
    id: "1",
    type: "input",
    data: { label: "File" },
    position: { x: 100, y: 100 },
    style: {
      background: "#e1f5fe",
      border: "2px solid #0288d1",
      borderRadius: "8px",
      padding: "10px",
      width: 120,
    },
  },
  {
    id: "2",
    data: { label: "Chunk" },
    position: { x: 300, y: 100 },
    style: {
      background: "#f3e5f5",
      border: "2px solid #7b1fa2",
      borderRadius: "8px",
      padding: "10px",
      width: 120,
    },
  },
  {
    id: "3",
    data: { label: "Embed" },
    position: { x: 500, y: 100 },
    style: {
      background: "#e8f5e8",
      border: "2px solid #388e3c",
      borderRadius: "8px",
      padding: "10px",
      width: 120,
    },
  },
  {
    id: "4",
    type: "output",
    data: { label: "Index" },
    position: { x: 700, y: 100 },
    style: {
      background: "#fff3e0",
      border: "2px solid #f57c00",
      borderRadius: "8px",
      padding: "10px",
      width: 120,
    },
  },
];

const initialEdges: Edge[] = [
  { id: "e1-2", source: "1", target: "2" },
  { id: "e2-3", source: "2", target: "3" },
  { id: "e3-4", source: "3", target: "4" },
];

export default function Canvas({ onRun }: CanvasProps) {
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);

  const onConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge(params, eds)),
    [setEdges]
  );

  return (
    <div className="w-full h-64 border border-gray-300 rounded-lg bg-gray-50">
      <div className="flex justify-between items-center p-4 border-b border-gray-300">
        <h2 className="text-lg font-semibold text-gray-700">Pipeline Canvas</h2>
        <button
          onClick={onRun}
          className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
        >
          Run Pipeline
        </button>
      </div>
      <div className="h-48">
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          fitView
          attributionPosition="bottom-left"
        >
          <Controls />
          <MiniMap />
          <Background color="#aaa" gap={16} />
        </ReactFlow>
      </div>
    </div>
  );
}
