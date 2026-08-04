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
}>();

function masterLabel(name: string): string {
  return name.trim().slice(0, 1).toLowerCase() || "?";
}

defineEmits<{
  (e: "selectMaster", index: number): void;
}>();
</script>

<template>
  <div class="top-bar">
    <!-- System menu (logo button) in the corner -->
    <slot name="menu" />

    <!-- File info: stretches to fill -->
    <div class="panel file-info">
      <div class="file-path">
        {{ fontLabel || "No font loaded" }}
        <span v-if="demo" class="demo-badge">demo font</span>
      </div>
      <div v-if="demo" class="demo-note">
        This is the bundled demo — open your own font from the
        Runebender menu in the top-left corner.
      </div>
      <div
        v-else-if="fontLabel"
        class="save-status"
        :class="{ saved: !unsaved && lastSaved }"
      >
        <span class="save-state">{{ !unsaved && lastSaved ? `Saved ${lastSaved}` : "Not saved" }}</span>
        <span v-if="notice" class="notice" :title="notice"> · {{ notice }}</span>
        <span v-else-if="sourceLabel" class="source-label" :title="sourceLabel"> · {{ sourceLabel }}</span>
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
 *   PANEL_BACKGROUND       #1C1C1C
 *   PANEL_OUTLINE / BASE_F #606060
 *   PRIMARY_UI_TEXT / BASE_I #909090
 *   SECONDARY_UI_TEXT / BASE_G #707070
 *   GRID_CELL_SELECTED_OUTLINE / TOOLBAR_ICON_HOVERED #18B86F
 *   MARK_YELLOW (Not saved) #FFDD33
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
  background: var(--rb-panel-background, #1c1c1c);
  border: var(--rb-stroke-width, 1px) solid var(--rb-panel-outline, #606060);
  border-radius: var(--rb-panel-radius, 12px);
  display: flex;
  align-items: center;
}

.file-info {
  flex: 1;
  padding: 6px 12px;
  gap: 2px;
  flex-direction: column;
  align-items: flex-start;
  justify-content: center;
  min-width: 0;
}
.file-path {
  color: var(--rb-muted-text, #808080);
  font: var(--rb-ui-font-size, 13px) ui-sans-serif, system-ui, sans-serif;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  width: 100%;
  min-width: 0;
}
.demo-badge {
  display: inline-block;
  margin-left: 8px;
  padding: 1px 8px;
  border: var(--rb-stroke-width, 1px) solid var(--rb-accent, #18b86f);
  border-radius: 999px;
  color: var(--rb-accent, #18b86f);
  font: var(--rb-ui-font-size, 13px) ui-sans-serif, system-ui, sans-serif;
  vertical-align: 2px;
}
.demo-note {
  color: var(--rb-secondary-text, #707070);
  font: var(--rb-ui-font-size, 13px) ui-sans-serif, system-ui, sans-serif;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
}

.save-status {
  color: var(--rb-warning, #ffdc32);
  font: var(--rb-ui-font-size, 13px) ui-sans-serif, system-ui, sans-serif;
  display: flex;
  flex-shrink: 1;
  max-width: 100%;
  white-space: nowrap;
  min-width: 0;
}
.save-status.saved {
  color: var(--rb-accent, #18b86f);
}
.save-state {
  flex: 0 0 auto;
}
.notice {
  color: var(--rb-warning, #ffdc32);
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}
.source-label {
  color: var(--rb-secondary-text, #707070);
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
}

.masters {
  padding: 6px;
  gap: 6px;
}
.master-btn {
  appearance: none;
  font: inherit;
  background: var(--rb-button-background, #181818);
  color: var(--rb-glyph-preview, #808080);
  border: var(--rb-stroke-width, 1px) solid var(--rb-panel-outline, #606060);
  border-radius: var(--rb-button-radius, 8px);
  width: 48px;
  height: 48px;
  cursor: pointer;
  font: var(--rb-ui-font-size, 13px) ui-sans-serif, system-ui, sans-serif;
  display: flex;
  align-items: center;
  justify-content: center;
}
.master-btn:hover {
  color: var(--rb-accent, #18b86f);
}
.master-btn.active {
  color: var(--rb-accent, #18b86f);
  border-color: var(--rb-accent, #18b86f);
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
