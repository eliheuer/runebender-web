<script setup lang="ts">
// Left panel for Select mode, under the measure panel: curve-smoothness
// layers drawn by the wasm renderer's draw_curve_hud. Smoothness outranks
// popcount (see virtua-grotesk DESIGN.md) — this is how you see it.
const props = defineProps<{
  comb: boolean;
  continuity: boolean;
  tol: number;
}>();

const emit = defineEmits<{
  (e: "update:comb", v: boolean): void;
  (e: "update:continuity", v: boolean): void;
  (e: "update:tol", v: number): void;
  (e: "harmonize"): void;
  (e: "balance"): void;
  (e: "optimize"): void;
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
    <div v-if="props.continuity" class="legend">
      <div class="key"><span class="ring g2"></span>G2 · smooth</div>
      <div class="key"><span class="ring g1"></span>G1 · harmonize</div>
      <div class="key"><span class="ring line"></span>line↔curve</div>
      <div class="key"><span class="ring kink"></span>kink</div>
    </div>
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
    <button
      class="row-btn optimize"
      title="Auto-fair: optimize continuity, even curvature, and low popcount together"
      @click="emit('optimize')"
    >
      ✦ optimize
    </button>
    <div class="slider">
      <div class="slider-head">
        <span>tolerance</span>
        <span class="val">{{ Math.round(props.tol * 100) }}%</span>
      </div>
      <input
        type="range"
        min="0"
        max="0.4"
        step="0.02"
        :value="props.tol"
        title="How much local smoothness the optimizer may trade for cleaner popcount. Lower = stricter curves, fewer snaps. Higher = chases clean numbers harder."
        @input="emit('update:tol', Number(($event.target as HTMLInputElement).value))"
      />
      <div class="slider-ends"><span>smooth</span><span>clean</span></div>
    </div>
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
.legend {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 2px 2px 4px;
}
.key {
  display: flex;
  align-items: center;
  gap: 7px;
  font-size: 10px;
  opacity: 0.85;
}
.ring {
  width: 11px;
  height: 11px;
  border-radius: 50%;
  border: 1.75px solid;
  flex: none;
}
.ring.g2 {
  border-color: #21d4a6;
}
.ring.g1 {
  border-color: #ffd140;
}
.ring.line {
  border-color: #808ca3;
}
.ring.kink {
  border-color: #ff453b;
}
.slider {
  display: flex;
  flex-direction: column;
  gap: 3px;
  padding: 2px 2px 0;
}
.slider-head {
  display: flex;
  justify-content: space-between;
  font-size: 10px;
  opacity: 0.7;
}
.slider-head .val {
  opacity: 0.9;
  font-variant-numeric: tabular-nums;
}
.slider input[type="range"] {
  width: 100%;
  accent-color: var(--rb-accent, #18b86f);
}
.slider-ends {
  display: flex;
  justify-content: space-between;
  font-size: 9px;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  opacity: 0.4;
}
</style>
