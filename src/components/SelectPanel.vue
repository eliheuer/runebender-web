<script setup lang="ts">
// Left-side panel for Select mode: toggles for the live grid-measurement
// HUD layers (drawn by the wasm renderer's draw_measurements). Every toggle
// off returns the editor to its plain look — this panel is purely additive.
const props = defineProps<{
  colorize: boolean;
  handles: boolean;
  segments: boolean;
  spans: boolean;
  sidebearings: boolean;
}>();

const emit = defineEmits<{
  (e: "update:colorize", v: boolean): void;
  (e: "update:handles", v: boolean): void;
  (e: "update:segments", v: boolean): void;
  (e: "update:spans", v: boolean): void;
  (e: "update:sidebearings", v: boolean): void;
}>();

function setAll(value: boolean) {
  emit("update:colorize", value);
  emit("update:handles", value);
  emit("update:segments", value);
  emit("update:spans", value);
  emit("update:sidebearings", value);
}
</script>

<template>
  <section class="select-panel">
    <div class="label title">Measure</div>
    <!-- Two columns where both labels fit on one line; anything that
         would wrap takes the full width instead. -->
    <div class="grid">
      <button
        class="row-btn"
        :class="{ on: props.colorize }"
        title="Tint outline segments, curves, and handles by popcount"
        @click="emit('update:colorize', !props.colorize)"
      >
        colorize outline
      </button>
      <button
        class="row-btn"
        :class="{ on: props.handles }"
        title="Label Bézier handle lengths"
        @click="emit('update:handles', !props.handles)"
      >
        handle lengths
      </button>
      <button
        class="row-btn wide"
        :class="{ on: props.segments }"
        title="Label straight segment lengths"
        @click="emit('update:segments', !props.segments)"
      >
        segment lengths
      </button>
      <button
        class="row-btn wide"
        :class="{ on: props.spans }"
        title="Scan-line stem/counter/thickness spans with arrows"
        @click="emit('update:spans', !props.spans)"
      >
        stems &amp; counters
      </button>
      <button
        class="row-btn wide"
        :class="{ on: props.sidebearings }"
        title="Left/right side bearings + furthest-point columns"
        @click="emit('update:sidebearings', !props.sidebearings)"
      >
        side bearings
      </button>
      <button class="row-btn small" @click="setAll(true)">all on</button>
      <button class="row-btn small" @click="setAll(false)">all off</button>
    </div>
  </section>
</template>

<style scoped>
.select-panel {
  width: 138px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 8px;
  background: var(--rb-panel-bg, rgba(24, 24, 24, 0.92));
  border: 1px solid var(--rb-panel-border, rgba(255, 255, 255, 0.08));
  border-radius: 8px;
  pointer-events: auto;
}
.label {
  font-size: var(--rb-ui-label-size, 11px);
  letter-spacing: 0.02em;
  opacity: 0.5;
  margin-top: 2px;
}
.label.title {
  margin-top: 0;
  opacity: 0.7;
}
.row-btn {
  font: inherit;
  white-space: nowrap;
  font-size: var(--rb-ui-label-size, 11px);
  color: inherit;
  /* Buttons sit lighter than the panel they're on; typed-into fields
     sit darker. Same tokens as the rest of the editor chrome. */
  background: var(--rb-button-background, #1d1d1d);
  border: 1px solid var(--rb-panel-outline, #404040);
  border-radius: 6px;
  padding: 6px 8px;
  cursor: pointer;
  text-align: left;
}
.grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 6px;
}
.grid .wide {
  grid-column: 1 / -1;
}
.row-btn.small {
  padding: 4px 8px;
  opacity: 0.8;
  text-align: center;
}
.row-btn.on {
  background: color-mix(in srgb, var(--rb-accent, #18b86f) 22%, transparent);
  border-color: color-mix(in srgb, var(--rb-accent, #18b86f) 65%, transparent);
}
</style>
