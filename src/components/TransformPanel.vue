<script setup lang="ts">
import GeneratedIcon from "./GeneratedIcon.vue";

// Right-side transform panel. Mirrors runebender-xilem's
// `components/transform_panel.rs` as a compact 2-column action grid.
// Bounds display lives in CoordinatePanel, matching xilem's split
// between coordinate editing and transform actions.

type SelectionBounds = {
  count: number;
  x: number;
  y: number;
  width: number;
  height: number;
};

export type TransformActionId =
  | "flip-h"
  | "flip-v"
  | "rot-cw"
  | "rot-ccw"
  | "rot-180"
  | "duplicate"
  | "duplicate-repeat"
  | "union"
  | "subtract"
  | "intersect"
  | "exclude";

defineProps<{
  bounds?: SelectionBounds;
  contourCount: number;
  /** Whether the bottom text-preview pane is showing. */
  previewPaneVisible?: boolean;
}>();

const emit = defineEmits<{
  (e: "transform", action: TransformActionId): void;
  (e: "togglePreviewPane"): void;
}>();

const actions = [
  ["Flip Horizontal", "flip-h"],
  ["Flip Vertical", "flip-v"],
  ["Rotate 90 CW", "rot-cw"],
  ["Rotate 90 CCW", "rot-ccw"],
  ["Rotate 180", "rot-180"],
  ["Duplicate", "duplicate"],
  ["Dup + Repeat", "duplicate-repeat"],
  ["Union (Remove Overlap)", "union"],
  ["Subtract", "subtract"],
  ["Intersect", "intersect"],
  ["Exclude (XOR)", "exclude"],
] as const;

const ACTION_ICONS: Record<TransformActionId, string> = {
  "flip-h": "flip-h",
  "flip-v": "flip-v",
  "rot-cw": "rot-cw",
  "rot-ccw": "rot-ccw",
  "rot-180": "rot-cw",
  duplicate: "duplicate",
  "duplicate-repeat": "duplicate-repeat",
  union: "union",
  subtract: "subtract",
  intersect: "intersect",
  exclude: "exclude",
};

function actionEnabled(id: string, hasSelection: boolean, contourCount: number): boolean {
  if (["union", "subtract", "intersect", "exclude"].includes(id)) {
    return contourCount >= 2;
  }
  return hasSelection;
}

function actionImplemented(id: string): boolean {
  return (
    id === "flip-h" ||
    id === "flip-v" ||
    id === "rot-cw" ||
    id === "rot-ccw" ||
    id === "rot-180" ||
    id === "duplicate" ||
    id === "duplicate-repeat" ||
    id === "union" ||
    id === "subtract" ||
    id === "intersect" ||
    id === "exclude"
  );
}

function actionAvailable(
  id: string,
  hasSelection: boolean,
  contourCount: number,
): boolean {
  return actionImplemented(id) && actionEnabled(id, hasSelection, contourCount);
}

function runAction(
  id: TransformActionId,
  hasSelection: boolean,
  contourCount: number,
) {
  if (!actionAvailable(id, hasSelection, contourCount)) return;
  emit("transform", id);
}
</script>

<template>
  <section class="transform-panel" aria-label="Selection transforms">
    <div class="actions">
      <button
        v-for="[label, id] in actions"
        :key="id"
        type="button"
        class="action-btn"
        :class="{ disabled: !actionAvailable(id, !!bounds, contourCount) }"
        :title="label"
        :aria-label="label"
        :disabled="!actionAvailable(id, !!bounds, contourCount)"
        @click="runAction(id, !!bounds, contourCount)"
      >
        <GeneratedIcon :name="ACTION_ICONS[id]" />
      </button>
      <!-- The grid's spare cell: show/hide the bottom text preview.
           Parked here until it finds a better home. -->
      <button
        type="button"
        class="action-btn preview-toggle"
        :class="{ on: previewPaneVisible }"
        :title="previewPaneVisible ? 'Hide text preview' : 'Show text preview'"
        :aria-label="previewPaneVisible ? 'Hide text preview' : 'Show text preview'"
        :aria-pressed="!!previewPaneVisible"
        @click="emit('togglePreviewPane')"
      >
        <!-- Eye / eye-off drawn inline: the generated icon set (from
             the icons UFO) has no eye yet, and this control is a
             temporary home anyway. -->
        <svg viewBox="0 0 24 24" aria-hidden="true" class="eye">
          <path
            d="M12 5C5 5 1.5 12 1.5 12S5 19 12 19s10.5-7 10.5-7S19 5 12 5Z"
            fill="none"
            stroke="currentColor"
            stroke-width="1.6"
            stroke-linejoin="round"
          />
          <circle cx="12" cy="12" r="3.2" fill="none" stroke="currentColor" stroke-width="1.6" />
          <path
            v-if="!previewPaneVisible"
            d="M4 20 20 4"
            fill="none"
            stroke="currentColor"
            stroke-width="1.6"
            stroke-linecap="round"
          />
        </svg>
      </button>
    </div>
  </section>
</template>

<style scoped>
.transform-panel {
  width: 117px;
  box-sizing: border-box;
  background: var(--rb-panel-background);
  border: var(--rb-stroke-width) solid var(--rb-panel-outline);
  border-radius: var(--rb-panel-radius);
  padding: 6px;
  pointer-events: auto;
}

.actions {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 6px;
}

.action-btn .eye {
  width: 24px;
  height: 24px;
  display: block;
}

.action-btn.preview-toggle.on {
  color: var(--rb-accent);
  border-color: var(--rb-accent);
}

.action-btn {
  appearance: none;
  width: 48px;
  height: 48px;
  background: var(--rb-button-background);
  border: var(--rb-stroke-width) solid var(--rb-panel-outline);
  border-radius: var(--rb-button-radius);
  /* Neutral gray by default, green only on hover — same as the tool
     palette and xilem's transform buttons. Enabled buttons should not
     glow green just for being usable. */
  color: var(--rb-glyph-preview);
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}
.action-btn:not(.disabled):hover {
  color: var(--rb-accent);
  border-color: var(--rb-accent);
}
.action-btn.disabled {
  color: var(--rb-glyph-preview);
  opacity: 0.55;
}

</style>
