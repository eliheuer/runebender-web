<script setup lang="ts">
// Top file-info bar. Mirrors runebender-xilem's
//   views/glyph_grid/mod.rs `file_info_panel`
// + components/master_toolbar.rs
// + components/system_toolbar.rs
//
// Layout: system menu (logo button) in the far-left corner via the
// `menu` slot; font label + save status stretches; master switcher
// on the right. All tiles are individual panels, separated by
// BENTO_GAP (6px), matching xilem's bento layout.

defineProps<{
  /** Display label for the open font (UFO folder name, designspace
   *  path, or empty string when nothing's loaded). */
  fontLabel: string;
  /** True when an in-memory edit has not been serialized back. */
  unsaved?: boolean;
  /** Last successful save time, e.g. "03:42 PM". */
  lastSaved?: string | null;
  /** Source destination summary, e.g. linked disk root. */
  sourceLabel?: string | null;
  /** Workspace notice: save conflicts, external changes held back. */
  notice?: string | null;
  /** Names of available masters. Stubbed to a single entry until
   *  designspace loading lands (Phase 7). */
  masters?: string[];
  /** Index of the active master. */
  activeMaster?: number;
  /** Rendered preview glyphs for each master, usually lowercase n. */
  masterPreviews?: Array<string | undefined>;
  /** Show only the status/file panel, used above the editor canvas. */
  fileOnly?: boolean;
  /** The bundled demo font is loaded — label it as such and point at
   *  the system menu for opening a real source. */
  demo?: boolean;
  /** Saved text arrangements, in tab order. */
  textTabs?: string[];
  /** Open tab: null is the font overview. */
  activeTextTab?: number | null;
}>();

/** What a tab shows: its text, or a placeholder while it is empty. The
 *  tab itself ellipsises whatever does not fit its share of the row. */
function tabLabel(text: string): string {
  const trimmed = text.replace(/\s+/g, " ").trim();
  return trimmed || "Empty";
}

function masterLabel(name: string): string {
  return name.trim().slice(0, 1).toLowerCase() || "?";
}

defineEmits<{
  (e: "selectMaster", index: number): void;
  (e: "selectTextTab", index: number | null): void;
  (e: "addTextTab"): void;
  (e: "closeTextTab", index: number): void;
}>();
</script>

<template>
  <div class="top-bar">
    <!-- System menu (logo button) in the corner -->
    <slot name="menu" />

    <!-- File info and text tabs share one wide tile: the file name and
         save state on the top line, the tabs on the bottom. Two tiles
         here left the name cramped and the tabs stranded in the
         corner. -->
    <div class="panel file-info">
      <div class="file-line">
        <span class="file-path">
          {{ fontLabel || "No font loaded" }}
          <span v-if="demo" class="demo-badge">(demo font)</span>
        </span>
        <span v-if="demo" class="demo-note">
          This is the bundled demo — open your own font from the
          Runebender menu in the top-left corner.
        </span>
        <span
          v-else-if="fontLabel"
          class="save-status"
          :class="{ saved: !unsaved && lastSaved }"
        >
          <span class="save-state">{{ !unsaved && lastSaved ? `Saved ${lastSaved}` : "Not saved" }}</span>
          <span v-if="notice" class="notice" :title="notice"> · {{ notice }}</span>
          <span v-else-if="sourceLabel" class="source-label" :title="sourceLabel"> · {{ sourceLabel }}</span>
        </span>
      </div>

      <!-- The font overview plus every text you have been working on,
           the way Glyphs keeps them across the top. Tabs share the row
           evenly rather than bunching up on the left. -->
      <div v-if="fontLabel" class="text-tabs" role="tablist">
        <button
          type="button"
          role="tab"
          class="text-tab fixed"
          :class="{ active: activeTextTab === null }"
          :aria-selected="activeTextTab === null"
          title="Full glyph overview"
          @click="$emit('selectTextTab', null)"
        >
          Font
        </button>
        <button
          v-for="(text, index) in textTabs"
          :key="index"
          type="button"
          role="tab"
          class="text-tab"
          :class="{ active: activeTextTab === index }"
          :aria-selected="activeTextTab === index"
          :title="text || 'Empty tab'"
          @click="$emit('selectTextTab', index)"
        >
          <span class="text-tab-label">{{ tabLabel(text) }}</span>
          <span
            class="text-tab-close"
            role="button"
            aria-label="Close tab"
            @click.stop="$emit('closeTextTab', index)"
          >×</span>
        </button>
        <button
          type="button"
          class="text-tab fixed add"
          title="New text tab"
          aria-label="New text tab"
          @click="$emit('addTextTab')"
        >
          +
        </button>
      </div>
    </div>

    <!-- Master switcher -->
    <div v-if="!fileOnly && masters && masters.length > 1" class="panel masters">
      <button
        v-for="(name, i) in masters"
        :key="name"
        type="button"
        class="master-btn"
        :class="{ active: i === activeMaster }"
        :title="name"
        @click="$emit('selectMaster', i)"
      >
        <span
          v-if="masterPreviews?.[i]"
          class="master-preview"
          aria-hidden="true"
          v-html="masterPreviews[i]"
        />
        <span v-else>{{ masterLabel(name) }}</span>
      </button>
    </div>

  </div>
</template>

<style scoped>
/*
 * Colors from xilem/src/theme.rs:
 * Colour comes from themes/runebender.theme.json via the --rb-*
 * custom properties on the host element. Nothing here names a colour.
 *
 * Sizes:
 *   TOOLBAR_BUTTON_RADIUS  6px
 *   TOOLBAR_BORDER_WIDTH   1px
 *   BENTO_GAP              6px (parent grid)
 */

.top-bar {
  display: flex;
  gap: 6px;
  height: 64px; /* 48px toolbar item + 8px panel padding on each side */
  flex-shrink: 0;
}

.panel {
  background: var(--rb-panel-background);
  border: var(--rb-stroke-width) solid var(--rb-panel-outline);
  border-radius: var(--rb-panel-radius);
  display: flex;
  align-items: center;
}

.file-info {
  flex: 1;
  /* Padding, the gap between the two lines, and the space inside a tab
     are all in the same range, so the tile reads as evenly spaced from
     top to bottom. */
  padding: 7px 8px;
  gap: 6px;
  flex-direction: column;
  align-items: stretch;
  justify-content: center;
  min-width: 0;
}

/* Top line: the name, then how it stands with the disk. */
.file-line {
  display: flex;
  /* Takes whatever the tab row leaves, so the name sits in open space
     rather than pressed against the top of the tile. */
  flex: 1 1 auto;
  align-items: center;
  gap: 8px;
  min-width: 0;
  padding: 0 4px;
  line-height: 1.15;
  white-space: nowrap;
}
.file-path {
  color: var(--rb-muted-text);
  font: var(--rb-ui-font-size) ui-sans-serif, system-ui, sans-serif;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
  flex: 0 1 auto;
}
.demo-badge {
  /* Parenthesised rather than a pill: same information, much quieter
     beside the file name. */
  margin-left: 6px;
  color: var(--rb-accent);
}
.demo-note {
  color: var(--rb-secondary-text);
  font: var(--rb-ui-font-size) ui-sans-serif, system-ui, sans-serif;
  overflow: hidden;
  text-overflow: ellipsis;
  min-width: 0;
  flex: 0 1 auto;
}

.save-status {
  color: var(--rb-warning);
  font: var(--rb-ui-font-size) ui-sans-serif, system-ui, sans-serif;
  display: flex;
  flex: 0 1 auto;
  min-width: 0;
  white-space: nowrap;
}
.save-status.saved {
  color: var(--rb-accent);
}
.save-state {
  flex: 0 0 auto;
}
.notice {
  color: var(--rb-warning);
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}
.source-label {
  color: var(--rb-secondary-text);
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* Bottom line: the tabs, sharing the width of the tile. */
.text-tabs {
  display: flex;
  flex: 0 0 auto;
  height: 22px;
  align-items: stretch;
  gap: 4px;
  min-width: 0;
}
.text-tab {
  appearance: none;
  /* Every text tab takes an equal share of what the fixed ones leave. */
  flex: 1 1 0;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  min-width: 0;
  max-width: 260px;
  padding: 0 10px;
  background: var(--rb-button-background);
  border: var(--rb-stroke-width) solid var(--rb-panel-outline);
  border-radius: var(--rb-button-radius);
  color: var(--rb-primary-text);
  font: var(--rb-ui-font-size) ui-sans-serif, system-ui, sans-serif;
  cursor: pointer;
  white-space: nowrap;
}
/* Font and + say the same thing at any width, so they stay small. */
.text-tab.fixed {
  flex: 0 0 auto;
}
.text-tab:hover {
  color: var(--rb-accent);
  border-color: var(--rb-accent);
}
.text-tab.active {
  color: var(--rb-accent);
  border-color: var(--rb-accent);
}
.text-tab-label {
  overflow: hidden;
  text-overflow: ellipsis;
}
.text-tab-close {
  flex: 0 0 auto;
  opacity: 0.55;
}
.text-tab-close:hover {
  opacity: 1;
}
.text-tab.add {
  padding: 0 12px;
}

.masters {
  padding: 6px;
  gap: 6px;
}
.master-btn {
  appearance: none;
  font: inherit;
  background: var(--rb-button-background);
  color: var(--rb-glyph-preview);
  border: var(--rb-stroke-width) solid var(--rb-panel-outline);
  border-radius: var(--rb-button-radius);
  width: 48px;
  height: 48px;
  cursor: pointer;
  font: var(--rb-ui-font-size) ui-sans-serif, system-ui, sans-serif;
  display: flex;
  align-items: center;
  justify-content: center;
}
.master-btn:hover {
  color: var(--rb-accent);
}
.master-btn.active {
  color: var(--rb-accent);
  border-color: var(--rb-accent);
}

.master-preview {
  width: 32px;
  height: 32px;
  color: currentColor;
  display: flex;
  align-items: center;
  justify-content: center;
}
.master-preview :deep(svg) {
  width: 100%;
  height: 100%;
  display: block;
  overflow: visible;
}

</style>
