<script setup lang="ts">
// Left-side sketch panel: raster brush controls for the sketch layer.
// Brushes come in DESIGN-SYSTEM sizes (the stroke ladder) so a sketch is
// born with correct stroke weights — the tracer then only has to find
// the skeleton, not guess the weight.
const props = defineProps<{
  brush: number;
  erase: boolean;
  traceMode: string;
  hasInk: boolean;
  tracing: boolean;
  drafting: boolean;
  banking: boolean;
  bankCount: number | null;
  bankFlash: boolean;
}>();

const emit = defineEmits<{
  (e: "update:brush", v: number): void;
  (e: "update:erase", v: boolean): void;
  (e: "update:traceMode", v: string): void;
  (e: "clear"): void;
  (e: "trace"): void;
  (e: "draft"): void;
  (e: "bank"): void;
}>();

// Stroke-ladder brush presets (design units): lc/cap stems + bars, both
// masters, plus a fine liner for construction marks.
const BRUSHES = [16, 80, 96, 104, 152, 168, 192, 200];
const MODES = [
  ["default", "corners"],
  ["smooth", "smooth"],
];
</script>

<template>
  <section class="sketch-panel">
    <div class="label title">sketch</div>
    <div class="label">brush</div>
      <div class="brush-grid">
        <button
          v-for="b in BRUSHES"
          :key="b"
          class="brush-btn"
          :class="{ on: props.brush === b && !props.erase }"
          :title="`${b} unit brush`"
          @click="
            emit('update:brush', b);
            emit('update:erase', false);
          "
        >
          {{ b }}
        </button>
      </div>
      <button
        class="row-btn small"
        :class="{ on: props.erase }"
        @click="emit('update:erase', !props.erase)"
      >
        erase
      </button>
      <button class="row-btn small" :disabled="!props.hasInk" @click="emit('clear')">
        clear
      </button>

      <div class="label">trace</div>
      <div class="brush-grid two">
        <button
          v-for="[m, label] in MODES"
          :key="m"
          class="brush-btn"
          :class="{ on: props.traceMode === m }"
          @click="emit('update:traceMode', m)"
        >
          {{ label }}
        </button>
      </div>
      <button
        class="row-btn trace"
        :disabled="!props.hasInk || props.tracing || props.drafting"
        @click="emit('trace')"
      >
        {{ props.tracing ? "tracing…" : "Trace → draft" }}
      </button>
      <button
        class="row-btn trace virtua"
        :disabled="!props.hasInk || props.tracing || props.drafting"
        @click="emit('draft')"
      >
        {{ props.drafting ? "drafting…" : "Draft with Virtua" }}
      </button>
      <button
        class="row-btn small"
        :class="{ flash: props.bankFlash }"
        :disabled="!props.hasInk || props.banking"
        title="save (this sketch -> this glyph's outline) as a training pair"
        @click="emit('bank')"
      >
        {{
          props.banking
            ? "banking…"
            : props.bankFlash
              ? `banked ✓ ${props.bankCount ?? ""}`
              : props.bankCount != null
                ? `bank pair (${props.bankCount})`
                : "bank pair"
        }}
      </button>
  </section>
</template>

<style scoped>
.sketch-panel {
  width: 117px;
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
  font-size: 10px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  opacity: 0.5;
  margin-top: 2px;
}
.brush-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 4px;
}
.row-btn,
.brush-btn {
  font: inherit;
  font-size: 11px;
  color: inherit;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  padding: 6px 4px;
  cursor: pointer;
}
.row-btn.small {
  padding: 4px;
}
.row-btn.on,
.brush-btn.on {
  background: color-mix(in srgb, var(--rb-accent, #18b86f) 22%, transparent);
  border-color: color-mix(in srgb, var(--rb-accent, #18b86f) 65%, transparent);
}
.label.title {
  margin-top: 0;
  opacity: 0.7;
}
.row-btn:disabled {
  opacity: 0.4;
  cursor: default;
}
.row-btn.trace {
  margin-top: 2px;
}
.row-btn.virtua {
  border-color: color-mix(in srgb, var(--rb-accent, #18b86f) 45%, transparent);
}
.row-btn.flash {
  background: color-mix(in srgb, var(--rb-accent, #18b86f) 45%, transparent);
  border-color: var(--rb-accent, #18b86f);
  transition: background 0.15s;
}
</style>
