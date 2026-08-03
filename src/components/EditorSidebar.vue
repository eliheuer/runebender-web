<script setup lang="ts">
// The Glyphs-style left sidebar in editor mode: one bento tile with
// three tabs — mini glyph overview, the current glyph's shapes, and
// the variation axes. Parity target is easy app-switching for Glyphs
// users, not a pixel copy (see the Glyphs 4 tab bar for reference).

import { computed, ref } from "vue";
import GlyphCell from "./GlyphCell.vue";

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
  }>(),
  {
    glyphs: () => [],
    currentGlyph: "",
    shapes: () => [],
    axes: () => [],
    masters: () => [],
    activeMaster: 0,
  },
);

const emit = defineEmits<{
  (e: "jumpGlyph", name: string): void;
  (e: "selectShape", shape: SidebarShape): void;
  (e: "selectMaster", index: number): void;
  (e: "backToGrid"): void;
}>();

const tab = ref<"overview" | "shapes" | "axes">("overview");
const search = ref("");

const filteredGlyphs = computed(() => {
  const query = search.value.trim().toLowerCase();
  if (!query) return props.glyphs;
  return props.glyphs.filter((g) => g.name.toLowerCase().includes(query));
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

    <div v-if="tab === 'overview'" class="tab-body">
      <button
        type="button"
        class="back-to-grid"
        title="Open the full glyph overview"
        @click="emit('backToGrid')"
      >
        ⊞ Full glyph overview
      </button>
      <input
        v-model="search"
        class="search"
        type="search"
        placeholder="Search glyphs…"
        aria-label="Search glyphs"
      />
      <div class="mini-grid">
        <GlyphCell
          v-for="item in filteredGlyphs"
          :key="item.name"
          :name="item.name"
          :unicode="item.unicode"
          :svg="item.svg"
          :selected="item.name === currentGlyph"
          :column-span="item.columnSpan"
          :mark-color="item.markColor"
          @click="emit('jumpGlyph', item.name)"
        />
      </div>
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
      <div class="side-label">Font axes</div>
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
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--rb-secondary-text, #707070);
}

.search {
  box-sizing: border-box;
  width: 100%;
  padding: 7px 9px;
  background: var(--rb-button-background, #181818);
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

/* The mini grid reuses the main overview's GlyphCell, scaled down:
   same mark-color outlines, name + unicode labels, wide glyphs span
   extra columns — just smaller. */
.mini-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 4px;
}
.mini-grid :deep(.cell) {
  height: 82px;
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
