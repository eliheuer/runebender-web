<script setup lang="ts">
// The Glyphs-style left sidebar in editor mode: one bento tile with
// three tabs — mini glyph overview, the current glyph's shapes, and
// the variation axes. Parity target is easy app-switching for Glyphs
// users, not a pixel copy (see the Glyphs 4 tab bar for reference).

import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import GlyphCell from "./GlyphCell.vue";
import { MARK_COLORS, rgbaToCss } from "./markColors";

export type SidebarGlyphItem = {
  name: string;
  unicode?: string;
  svg?: string;
  columnSpan: number;
  markColor?: string;
};
export type SidebarShape = {
  label: string;
  detail: string;
  x: number;
  y: number;
  kind: "contour" | "component";
};
export type SidebarAxis = {
  name: string;
  tag: string;
  min: number;
  max: number;
  default: number;
};

const props = withDefaults(
  defineProps<{
    glyphs?: SidebarGlyphItem[];
    currentGlyph?: string;
    shapes?: SidebarShape[];
    axes?: SidebarAxis[];
    masters?: string[];
    activeMaster?: number;
    /** Mark color on the glyph open in the editor, if any. */
    markColor?: string;
    canApplyAllMasters?: boolean;
    markApplyAllMasters?: boolean;
  }>(),
  {
    glyphs: () => [],
    currentGlyph: "",
    shapes: () => [],
    axes: () => [],
    masters: () => [],
    activeMaster: 0,
    markColor: "",
    canApplyAllMasters: false,
    markApplyAllMasters: false,
  },
);

const emit = defineEmits<{
  (e: "jumpGlyph", name: string): void;
  (e: "selectShape", shape: SidebarShape): void;
  (e: "selectMaster", index: number): void;
  (e: "backToGrid"): void;
  (e: "setMark", rgba: string): void;
  (e: "update:markApplyAllMasters", value: boolean): void;
}>();

const tab = ref<"overview" | "shapes" | "axes">("overview");
const search = ref("");

const filteredGlyphs = computed(() => {
  const query = search.value.trim().toLowerCase();
  if (!query) return props.glyphs;
  return props.glyphs.filter((g) => g.name.toLowerCase().includes(query));
});

const MINI_COLUMNS = 4;
const MINI_GAP = 4;
const MINI_ROW_TARGET = 82;

// Rows are sized to divide the visible height evenly, the same trick
// the main grid uses (glyphGridRowHeight in Runebender.vue), so a row
// is never left half-showing at the bottom of the pane.
const scrollEl = ref<HTMLElement | null>(null);
const scrollHeight = ref(0);
let observer: ResizeObserver | undefined;

onMounted(() => {
  if (!scrollEl.value) return;
  observer = new ResizeObserver(([entry]) => {
    scrollHeight.value = entry.contentRect.height;
  });
  observer.observe(scrollEl.value);
  scrollHeight.value = scrollEl.value.clientHeight;
});
onBeforeUnmount(() => observer?.disconnect());

const miniRowHeight = computed(() => {
  const height = scrollHeight.value;
  if (height <= 0) return MINI_ROW_TARGET;
  const rows = Math.max(
    1,
    Math.round((height + MINI_GAP) / (MINI_ROW_TARGET + MINI_GAP)),
  );
  return (height - MINI_GAP * (rows - 1)) / rows;
});

// Every row fills all four columns: pack greedily, then hand any
// leftover columns back to the cells in that row (widest first) so
// the grid never leaves a ragged gap at the end of a line.
const packedGlyphs = computed<SidebarGlyphItem[]>(() => {
  const out: SidebarGlyphItem[] = [];
  let row: SidebarGlyphItem[] = [];
  let used = 0;

  const flushRow = () => {
    if (!row.length) return;
    let left = MINI_COLUMNS - used;
    // Round-robin from the widest cell down, so growth lands where a
    // glyph or its name actually needs the room.
    const order = [...row].sort((a, b) => b.columnSpan - a.columnSpan);
    for (let i = 0; left > 0; i = (i + 1) % order.length) {
      order[i].columnSpan += 1;
      left -= 1;
    }
    out.push(...row);
    row = [];
    used = 0;
  };

  for (const glyph of filteredGlyphs.value) {
    const span = Math.min(MINI_COLUMNS, Math.max(1, glyph.columnSpan));
    if (used + span > MINI_COLUMNS) flushRow();
    row.push({ ...glyph, columnSpan: span });
    used += span;
    if (used === MINI_COLUMNS) flushRow();
  }
  flushRow();
  return out;
});
</script>

<template>
  <div class="editor-sidebar">
    <div class="tabs" role="tablist">
      <button
        type="button"
        role="tab"
        :aria-selected="tab === 'overview'"
        :class="{ active: tab === 'overview' }"
        title="Glyph overview"
        @click="tab = 'overview'"
      >
        ⊞
      </button>
      <button
        type="button"
        role="tab"
        :aria-selected="tab === 'shapes'"
        :class="{ active: tab === 'shapes' }"
        title="Shapes in this glyph"
        @click="tab = 'shapes'"
      >
        ⧉
      </button>
      <button
        type="button"
        role="tab"
        :aria-selected="tab === 'axes'"
        :class="{ active: tab === 'axes' }"
        title="Variation axes"
        @click="tab = 'axes'"
      >
        ⬌
      </button>
    </div>

    <div v-if="tab === 'overview'" class="tab-body overview">
      <button
        type="button"
        class="back-to-grid"
        title="Open the full glyph overview"
        @click="emit('backToGrid')"
      >
        ⊞ Full Glyph Overview
      </button>
      <!-- The grid takes all the height the fixed rows leave and
           scrolls on its own; search sits at the bottom, next to the
           colors it filters against. -->
      <div ref="scrollEl" class="mini-grid-scroll">
        <div
          class="mini-grid"
          :style="{ gridAutoRows: `${miniRowHeight}px` }"
        >
          <GlyphCell
            v-for="item in packedGlyphs"
            :key="item.name"
            :name="item.name"
            :unicode="item.unicode"
            :svg="item.svg"
            :selected="item.name === currentGlyph"
            :column-span="item.columnSpan"
            :mark-color="item.markColor"
            bare-unicode
            @click="emit('jumpGlyph', item.name)"
          />
        </div>
      </div>
      <hr class="rule" />
      <!-- Colors: a plain swatch row, no panel frame — the rule above
           is the only separation it needs inside this tile. -->
      <div class="colors-head">
        <span class="side-label">Colors</span>
        <button
          v-if="canApplyAllMasters"
          type="button"
          class="all-masters"
          :class="{ active: markApplyAllMasters }"
          title="Apply color changes to every master"
          @click="emit('update:markApplyAllMasters', !markApplyAllMasters)"
        >
          All Masters
        </button>
      </div>
      <div class="swatch-row">
        <button
          v-for="color in MARK_COLORS"
          :key="color.rgba"
          type="button"
          class="swatch"
          :class="{ active: color.rgba === markColor }"
          :style="{ background: rgbaToCss(color.rgba), color: rgbaToCss(color.rgba) }"
          :title="color.name"
          :aria-label="`Set mark color: ${color.name}`"
          :disabled="!currentGlyph"
          @click="emit('setMark', color.rgba)"
        />
        <button
          type="button"
          class="swatch clear"
          title="Clear mark color"
          aria-label="Clear mark color"
          :disabled="!currentGlyph"
          @click="emit('setMark', '')"
        >
          ✕
        </button>
      </div>
      <input
        v-model="search"
        class="search"
        type="search"
        placeholder="Search Glyphs…"
        aria-label="Search glyphs"
      />
    </div>

    <div v-else-if="tab === 'shapes'" class="tab-body">
      <div class="side-label">Shapes · {{ currentGlyph || "none" }}</div>
      <button
        v-for="(shape, index) in shapes"
        :key="`${shape.kind}-${index}`"
        type="button"
        class="shape-row"
        @click="emit('selectShape', shape)"
      >
        <span class="shape-kind">{{ shape.kind === "component" ? "◇" : "◌" }}</span>
        <span class="shape-label">{{ shape.label }}</span>
        <span class="shape-detail">{{ shape.detail }}</span>
      </button>
      <div v-if="!shapes.length" class="hint">No shapes in this glyph yet.</div>
    </div>

    <div v-else class="tab-body">
      <div class="side-label">Font Axes</div>
      <template v-if="axes.length">
        <div v-for="axis in axes" :key="axis.tag" class="axis">
          <div class="axis-head">
            <span>{{ axis.name }}</span>
            <span class="axis-range">{{ axis.min }}–{{ axis.max }}</span>
          </div>
          <input
            type="range"
            :min="axis.min"
            :max="axis.max"
            :value="axis.default"
            disabled
            :title="`${axis.tag} — live interpolation is coming; sliders unlock with it`"
          />
        </div>
      </template>
      <div v-else class="hint">
        No variation axes — open a designspace or .glyphs source with
        more than one master.
      </div>
      <div class="side-label">Masters</div>
      <button
        v-for="(master, index) in masters"
        :key="master"
        type="button"
        class="master-row"
        :class="{ active: index === activeMaster }"
        @click="emit('selectMaster', index)"
      >
        <span class="dot" />
        <span>{{ master }}</span>
      </button>
      <div class="hint">
        Live interpolation between masters lands here next — sliders
        preview any instance, click a master to edit it.
      </div>
    </div>
  </div>
</template>

<style scoped>
.editor-sidebar {
  box-sizing: border-box;
  width: 232px;
  min-height: 220px;
  max-height: 46vh;
  display: flex;
  flex-direction: column;
  background: var(--rb-panel-background, #1c1c1c);
  border: var(--rb-stroke-width, 1px) solid var(--rb-panel-outline, #606060);
  border-radius: var(--rb-panel-radius, 12px);
  overflow: hidden;
}

.tabs {
  display: flex;
  gap: 4px;
  padding: 6px;
  border-bottom: 1px solid rgba(96, 96, 96, 0.5);
  flex: 0 0 auto;
}
.tabs button {
  flex: 1;
  height: 32px;
  border: 1px solid transparent;
  border-radius: var(--rb-button-radius, 8px);
  background: transparent;
  color: var(--rb-secondary-text, #707070);
  font-size: 15px;
  cursor: pointer;
}
.tabs button.active {
  color: var(--rb-accent, #18b86f);
  border-color: var(--rb-panel-outline, #606060);
  background: var(--rb-button-background, #181818);
}

.tab-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.side-label {
  font: 11px ui-sans-serif, system-ui, sans-serif;
  letter-spacing: 0.02em;
  color: var(--rb-secondary-text, #707070);
}

.search {
  /* Extra air above: the swatch row is visually dense, so the search
     field needs more separation than the tile's default gap. */
  margin-top: 6px;
  box-sizing: border-box;
  width: 100%;
  padding: 7px 9px;
  /* Typed-into fields sit darker than the panel; buttons sit lighter
     (see .back-to-grid). Same split as the font-info fields. */
  background: var(--rb-app-background, #0c0c0c);
  border: 1px solid var(--rb-panel-outline, #606060);
  border-radius: var(--rb-button-radius, 8px);
  color: var(--rb-primary-text, #909090);
  font: 13px ui-sans-serif, system-ui, sans-serif;
}

.back-to-grid {
  box-sizing: border-box;
  width: 100%;
  padding: 7px 9px;
  background: var(--rb-button-background, #181818);
  border: 1px solid var(--rb-panel-outline, #606060);
  border-radius: var(--rb-button-radius, 8px);
  color: var(--rb-primary-text, #909090);
  font: 13px ui-sans-serif, system-ui, sans-serif;
  cursor: pointer;
  text-align: left;
}
.back-to-grid:hover {
  color: var(--rb-accent, #18b86f);
  border-color: var(--rb-accent, #18b86f);
}

/* The overview tab owns its own scrolling: the grid stretches, the
   back button, colors and search stay pinned. */
.tab-body.overview {
  overflow: hidden;
  /* The overview button sits on the grid: same side padding, same
     4px gap as the rows below it. Sections further down add their
     own breathing room. */
  gap: 4px;
}
.mini-grid-scroll {
  flex: 1 1 auto;
  min-height: 0;
  overflow-y: auto;
  /* Same rule as the main grid: rows snap so a cell is never left
     sliced in half at the top or bottom edge. */
  scroll-snap-type: y mandatory;
  /* No scrollbar track — it would eat into the right margin only and
     leave the grid visibly off-center in the tile. */
  scrollbar-width: none;
}
.mini-grid-scroll::-webkit-scrollbar {
  display: none;
}

.rule {
  flex: 0 0 auto;
  margin-top: 6px;
  width: 100%;
  height: 0;
  margin: 0;
  border: none;
  border-top: 1px solid rgba(96, 96, 96, 0.5);
}

.colors-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
}
.all-masters {
  padding: 0;
  background: none;
  border: none;
  color: var(--rb-secondary-text, #707070);
  font: 11px ui-sans-serif, system-ui, sans-serif;
  letter-spacing: 0.02em;
  cursor: pointer;
}
.all-masters.active,
.all-masters:hover {
  color: var(--rb-accent, #18b86f);
}

.swatch-row {
  /* Spread the eight buttons edge to edge so the row's side margins
     match the rest of the tile instead of trailing off to the right. */
  display: flex;
  justify-content: space-between;
  gap: 4px;
}
.swatch-row .swatch {
  appearance: none;
  width: 18px;
  height: 18px;
  padding: 0;
  border-radius: 50%;
  border: none;
  cursor: pointer;
}
.swatch-row .swatch:disabled {
  opacity: 0.45;
  cursor: default;
}
.swatch-row .swatch.active {
  /* The selection is one solid ring in the swatch's own color,
     separated from the disc by the tile background. */
  box-shadow:
    0 0 0 2px var(--rb-panel-background, #1c1c1c),
    0 0 0 3px currentColor;
}
.swatch-row .swatch:hover:not(:disabled) {
  box-shadow:
    0 0 0 2px var(--rb-panel-background, #1c1c1c),
    0 0 0 3px var(--rb-mark-hover-ring, #bbbbbb);
}
.swatch-row .swatch.clear {
  /* A filled disc like the others — an outlined one reads smaller. */
  background: var(--rb-panel-outline, #606060);
  border-color: transparent;
  color: var(--rb-panel-background, #1c1c1c);
  font-size: 11px;
  line-height: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}
.swatch-row .swatch.clear:hover:not(:disabled) {
  color: var(--rb-accent, #18b86f);
}

/* The mini grid reuses the main overview's GlyphCell, scaled down:
   same mark-color outlines, name + unicode labels, wide glyphs span
   extra columns — just smaller. */
.mini-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 4px;
}
.mini-grid :deep(.cell) {
  height: 100%;
  border-radius: 6px;
}
.mini-grid :deep(.cell-glyph) {
  flex: 1 1 auto;
  min-height: 0;
  padding: 5px 4px 2px;
}
.mini-grid :deep(.cell-labels) {
  min-height: auto;
  padding: 2px 4px 4px;
  gap: 0;
}
.mini-grid :deep(.cell .cell-name),
.mini-grid :deep(.cell .cell-unicode) {
  font-size: 9px;
  line-height: 1.3;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.shape-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 8px;
  background: transparent;
  border: none;
  border-radius: var(--rb-button-radius, 8px);
  color: var(--rb-primary-text, #909090);
  font: 13px ui-sans-serif, system-ui, sans-serif;
  cursor: pointer;
  text-align: left;
}
.shape-row:hover {
  background: var(--rb-button-background, #181818);
  color: var(--rb-accent, #18b86f);
}
.shape-kind {
  color: var(--rb-secondary-text, #707070);
}
.shape-label {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.shape-detail {
  color: var(--rb-secondary-text, #707070);
  font-variant-numeric: tabular-nums;
}

.axis-head {
  display: flex;
  justify-content: space-between;
  color: #a9a9a9;
  font: 13px ui-sans-serif, system-ui, sans-serif;
}
.axis-range {
  color: var(--rb-secondary-text, #707070);
  font-variant-numeric: tabular-nums;
}
.axis input[type="range"] {
  width: 100%;
  accent-color: var(--rb-accent, #18b86f);
}

.master-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 9px;
  background: var(--rb-button-background, #181818);
  border: 1px solid var(--rb-panel-outline, #606060);
  border-radius: var(--rb-button-radius, 8px);
  color: var(--rb-primary-text, #909090);
  font: 13px ui-sans-serif, system-ui, sans-serif;
  cursor: pointer;
  text-align: left;
}
.master-row .dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--rb-panel-outline, #606060);
}
.master-row.active {
  border-color: var(--rb-accent, #18b86f);
}
.master-row.active .dot {
  background: var(--rb-accent, #18b86f);
}

.hint {
  font: 12px/1.45 ui-sans-serif, system-ui, sans-serif;
  color: var(--rb-secondary-text, #707070);
  border: 1px dashed rgba(96, 96, 96, 0.6);
  border-radius: var(--rb-button-radius, 8px);
  padding: 8px 9px;
}
</style>
