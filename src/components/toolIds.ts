// Shared tool identifier strings. Matches runebender-xilem's
// `tools::ToolId` enum so a future runebender-core::tools module
// can use them directly.

export type ToolId =
  | "Select"
  | "Pen"
  | "HyperPen"
  | "Preview"
  | "Knife"
  | "Measure"
  | "Shapes"
  | "Sketch"
  | "Text";

export const TOOL_IDS: ToolId[] = [
  "Select",
  "Pen",
  "HyperPen",
  "Knife",
  "Measure",
  "Shapes",
  "Sketch",
  "Preview",
  "Text",
];

export const TOOL_LABELS: Record<ToolId, string> = {
  Select: "Select (V)",
  Pen: "Pen (P)",
  HyperPen: "Hyper Pen (H)",
  Preview: "Preview (Space)",
  Knife: "Knife (K)",
  Measure: "Measure",
  Shapes: "Shapes",
  Sketch: "Sketch",
  Text: "Text (T)",
};
