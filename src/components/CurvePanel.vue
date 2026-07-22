<script setup lang="ts">
// Left panel for Select mode, under the measure panel: curve-smoothness
// layers drawn by the wasm renderer's draw_curve_hud. Smoothness outranks
// popcount (see virtua-grotesk DESIGN.md) — this is how you see it.
const props = defineProps<{
  comb: boolean;
  continuity: boolean;
}>();

const emit = defineEmits<{
  (e: "update:comb", v: boolean): void;
  (e: "update:continuity", v: boolean): void;
  (e: "harmonize"): void;
  (e: "balance"): void;
}>();

function allOff() {
  emit("update:comb", false);
  emit("update:continuity", false);
}
</script>

<template>
  <section class="curve-panel">
    <div class="label title">curves</div>
    <button
      class="row-btn"
      :class="{ on: props.comb }"
      title="Speedpunk-style curvature comb"
      @click="emit('update:comb', !props.comb)"
    >
      curvature comb
    </button>
    <button
      class="row-btn"
      :class="{ on: props.continuity }"
      title="Continuity dot per smooth node: green G2, yellow G1 (harmonize), blue line↔curve, red kink"
      @click="emit('update:continuity', !props.continuity)"
    >
      continuity G0–G3
    </button>
    <button class="row-btn small" @click="allOff">all off</button>
    <div class="label">tools</div>
    <button
      class="row-btn"
      title="Harmonize selected smooth nodes (or all) to G2 curvature continuity"
      @click="emit('harmonize')"
    >
      harmonize → G2
    </button>
    <button
      class="row-btn"
      title="Balance selected segment handles (Tunni)"
      @click="emit('balance')"
    >
      balance handles
    </button>
  </section>
</template>

<style scoped>
.curve-panel {
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
  font-size: 10px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  opacity: 0.5;
  margin-top: 2px;
}
.label.title {
  margin-top: 0;
  opacity: 0.7;
}
.row-btn {
  font: inherit;
  font-size: 11px;
  color: inherit;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 6px;
  padding: 6px 8px;
  cursor: pointer;
  text-align: left;
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
