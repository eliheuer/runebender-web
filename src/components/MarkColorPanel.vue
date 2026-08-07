<script setup lang="ts">
// Mark-color picker. Mirrors runebender-xilem's
// `components/mark_color_panel.rs` — seven preset swatches in the
// same order as xilem's `theme::mark::COLORS`, plus a clear (X)
// button. The shared palette + RGBA-to-CSS conversion live in
// `markColors.ts` (Vue's <script setup> can't have `export`).
//
// Clicking a swatch tags the *selected* glyph (not the one in the
// editor — the cell that's highlighted in the grid). The active
// swatch is the one that matches the selected glyph's current
// mark color.

import { MARK_COLORS, cssForLabel } from "./markColors";

defineProps<{
  /** Currently-applied mark color on the selected glyph, or empty
   *  when no glyph is selected / glyph is unmarked. */
  active?: string;
  /** Apply the next mark-color edit to every master instead of only
   *  the active one. */
  applyAllMasters?: boolean;
  /** Hide the all-masters control for single-master fonts. */
  canApplyAllMasters?: boolean;
  /** False when no glyph is selected — swatches are visible but
   *  inert (a click would have nothing to apply to). */
  enabled?: boolean;
}>();

defineEmits<{
  /** Empty string means "clear the mark." */
  (e: "set", label: string): void;
  (e: "update:applyAllMasters", value: boolean): void;
}>();
</script>

<template>
  <div class="mark-color-panel" :class="{ disabled: !enabled }">
    <div class="header-row">
      <div class="header">Colors</div>
      <label
        v-if="canApplyAllMasters"
        class="all-masters-toggle"
        :class="{ active: applyAllMasters }"
        title="Apply color changes to all masters"
      >
        <input
          type="checkbox"
          :checked="applyAllMasters"
          @change="$emit('update:applyAllMasters', ($event.target as HTMLInputElement).checked)"
        />
        <span class="toggle-track" aria-hidden="true">
          <span class="toggle-thumb" />
        </span>
        <span>All masters</span>
      </label>
    </div>
    <div class="swatches">
      <button
        v-for="c in MARK_COLORS"
        :key="c.name"
        type="button"
        class="swatch"
        :class="{ active: c.name === active }"
        :style="{ background: cssForLabel(c.name) }"
        :title="c.name"
        :aria-label="`Set mark color: ${c.name}`"
        :disabled="!enabled"
        @click="$emit('set', c.name)"
      />
      <button
        type="button"
        class="swatch clear"
        title="Clear mark color"
        aria-label="Clear mark color"
        :disabled="!enabled"
        @click="$emit('set', '')"
      >
        <svg viewBox="0 0 16 16" aria-hidden="true">
          <line x1="4" y1="4" x2="12" y2="12" />
          <line x1="12" y1="4" x2="4" y2="12" />
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
/*
 * Colors from xilem theme.rs:
 * Colour comes from themes/runebender.theme.json via the --rb-*
 * custom properties on the host element. Nothing here names a colour.
 */

.mark-color-panel {
  width: 100%;
  min-height: 76px;
  box-sizing: border-box;
  background: var(--rb-panel-background);
  border: var(--rb-stroke-width) solid var(--rb-panel-outline);
  border-radius: var(--rb-panel-radius);
  padding: 0;
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}
.mark-color-panel.disabled {
  opacity: 0.5;
}

.header-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 10px 12px 8px;
}

.header {
  color: var(--rb-accent);
  font: var(--rb-ui-font-size) ui-sans-serif, system-ui, sans-serif;
  font-weight: 400;
  line-height: 16px;
}

.all-masters-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--rb-muted-text);
  font: var(--rb-ui-font-size) ui-sans-serif, system-ui, sans-serif;
  line-height: 18px;
  cursor: pointer;
  user-select: none;
  white-space: nowrap;
}
.all-masters-toggle input {
  position: absolute;
  opacity: 0;
  pointer-events: none;
}
.toggle-track {
  width: 22px;
  height: 12px;
  box-sizing: border-box;
  border: var(--rb-stroke-width) solid var(--rb-panel-outline);
  border-radius: 999px;
  background: var(--rb-panel-background);
  display: flex;
  align-items: center;
  flex: 0 0 auto;
  padding: 1px;
  transform: translateY(1px);
}
.toggle-thumb {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--rb-secondary-text);
  transform: translateX(0);
  transition: transform 0.08s, background 0.08s;
}
.all-masters-toggle.active {
  color: var(--rb-accent);
}
.all-masters-toggle.active .toggle-track {
  border-color: var(--rb-accent);
}
.all-masters-toggle.active .toggle-thumb {
  background: var(--rb-accent);
  transform: translateX(10px);
}

.swatches {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  justify-content: center;
  padding: 0 8px;
}

.swatch {
  appearance: none;
  width: 22px;
  height: 22px;
  border-radius: 50%;
  border: var(--rb-stroke-width) solid transparent;
  cursor: pointer;
  padding: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}
.swatch:hover:not(:disabled) {
  outline: var(--rb-stroke-width) solid var(--rb-mark-hover-ring);
  outline-offset: 0;
}
.swatch:disabled {
  cursor: default;
}

.swatch.clear {
  background: var(--rb-panel-background);
  border-color: var(--rb-panel-outline);
}
.swatch.clear svg {
  width: 12px;
  height: 12px;
  stroke: var(--rb-secondary-text);
  stroke-width: 1.5;
}
.swatch.clear:hover:not(:disabled) svg {
  stroke: var(--rb-accent);
}
</style>
