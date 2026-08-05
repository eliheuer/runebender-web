<script setup lang="ts">
// The system menu's panel: file actions (open/reopen/save) as a
// regular bento tile. In grid view the parent renders it at the top
// of the left column, pushing the category sidebar down; in editor
// view it hangs one bento gap below the menu button. Dismissal
// (outside click, Escape) is handled here; the parent owns the open
// state and the menu button toggles it.

import { onBeforeUnmount, onMounted, ref } from "vue";

withDefaults(
  defineProps<{
    saveEnabled?: boolean;
    /** Glyphs whose last save hit a disk conflict. */
    conflicts?: string[];
    saveAsEnabled?: boolean;
    closeEnabled?: boolean;
    /** Workspace remembered from a previous visit (FS Access host). */
    reopenName?: string | null;
    /** Recently opened folders / .glyphs files, newest first. */
    recents?: { index: number; name: string; kind: "folder" | "file" }[];
  }>(),
  {
    saveEnabled: false,
    conflicts: () => [],
    saveAsEnabled: false,
    closeEnabled: false,
    reopenName: null,
    recents: () => [],
  },
);

const emit = defineEmits<{
  (e: "newUfo"): void;
  (e: "newDesignspace"): void;
  (e: "openUfo"): void;
  (e: "openFolder"): void;
  (e: "openRecent", index: number): void;
  (e: "reopen"): void;
  (e: "save"): void;
  (e: "saveOverwriting"): void;
  (e: "saveAs"): void;
  (e: "closeEditor"): void;
  (e: "dismiss"): void;
}>();

function pickRecent(index: number) {
  emit("openRecent", index);
  emit("dismiss");
}

const rootEl = ref<HTMLElement | null>(null);

function pick(
  action:
    | "newUfo"
    | "newDesignspace"
    | "openUfo"
    | "openFolder"
    | "reopen"
    | "save"
    | "saveOverwriting"
    | "saveAs"
    | "closeEditor",
  enabled = true,
) {
  if (!enabled) return;
  emit(action);
  emit("dismiss");
}

function onWindowPointerDown(e: PointerEvent) {
  const target = e.target as Element | null;
  // Clicks on the menu button toggle the panel themselves; closing
  // here too would make the button reopen it in the same gesture.
  if (rootEl.value?.contains(target) || target?.closest(".system-menu")) {
    return;
  }
  emit("dismiss");
}

function onWindowKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    e.stopPropagation();
    emit("dismiss");
  }
}

onMounted(() => {
  window.addEventListener("pointerdown", onWindowPointerDown, {
    capture: true,
  });
  window.addEventListener("keydown", onWindowKeyDown, { capture: true });
});
onBeforeUnmount(() => {
  window.removeEventListener("pointerdown", onWindowPointerDown, {
    capture: true,
  });
  window.removeEventListener("keydown", onWindowKeyDown, { capture: true });
});
</script>

<template>
  <div ref="rootEl" class="system-menu-panel" role="menu">
    <button
      type="button"
      role="menuitem"
      title="Blank Regular UFO with the GF Latin Core glyph set and Google Fonts metadata"
      @click="pick('newUfo')"
    >
      New Font
    </button>
    <button
      type="button"
      role="menuitem"
      title="Same, as a designspace with Regular and Bold masters on a wght axis"
      @click="pick('newDesignspace')"
    >
      New Variable Font
    </button>
    <div class="separator" />
    <button
      type="button"
      role="menuitem"
      title="One dialog: highlight the folder holding your .designspace / .ufo / .glyphs and press Select"
      @click="pick('openFolder')"
    >
      Open Font Folder...
    </button>
    <button
      type="button"
      role="menuitem"
      title="Pick a .designspace or .glyphs file directly (a folder confirm follows for designspaces; .ufo is a folder — use Open Font Folder)"
      @click="pick('openUfo')"
    >
      Open Font File...
    </button>
    <template v-if="recents.length">
      <div class="separator" />
      <div class="group-label">Open Recent</div>
      <button
        v-for="entry in recents"
        :key="`recent-${entry.index}`"
        type="button"
        role="menuitem"
        :title="entry.kind === 'file' ? 'Glyphs file' : 'Font folder'"
        @click="pickRecent(entry.index)"
      >
        {{ entry.name }}{{ entry.kind === "folder" ? "/" : "" }}
      </button>
    </template>
    <button
      v-if="reopenName"
      type="button"
      role="menuitem"
      class="accent"
      @click="pick('reopen')"
    >
      Reopen {{ reopenName }}
    </button>
    <div class="separator" />
    <button
      type="button"
      role="menuitem"
      :disabled="!saveEnabled"
      @click="pick('save', saveEnabled)"
    >
      Save
    </button>
    <button
      v-if="conflicts.length"
      type="button"
      role="menuitem"
      class="accent"
      :title="`Overwrite the disk copy of ${conflicts.join(', ')} with your version`"
      @click="pick('saveOverwriting')"
    >
      Save Anyway ({{ conflicts.length }})
    </button>
    <button
      type="button"
      role="menuitem"
      :disabled="!saveAsEnabled"
      @click="pick('saveAs', saveAsEnabled)"
    >
      Save As...
    </button>
    <template v-if="closeEnabled">
      <div class="separator" />
      <button type="button" role="menuitem" @click="pick('closeEditor')">
        Close Editor
      </button>
    </template>
  </div>
</template>

<style scoped>
.system-menu-panel {
  box-sizing: border-box;
  padding: 6px;
  background: var(--rb-panel-background, #1c1c1c);
  border: var(--rb-stroke-width, 1px) solid var(--rb-panel-outline, #606060);
  border-radius: var(--rb-panel-radius, 12px);
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 0 0 auto;
}

.system-menu-panel button {
  appearance: none;
  text-align: left;
  padding: 8px 10px;
  background: transparent;
  border: none;
  border-radius: var(--rb-button-radius, 8px);
  color: var(--rb-primary-text, #909090);
  font: var(--rb-ui-font-size, 13px) ui-sans-serif, system-ui, sans-serif;
  cursor: pointer;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.system-menu-panel button:not(:disabled):hover {
  color: var(--rb-accent, #18b86f);
  background: var(--rb-button-background, #181818);
}
.system-menu-panel button:disabled {
  opacity: 0.55;
  cursor: default;
}
.system-menu-panel button.accent {
  color: var(--rb-accent, #18b86f);
}

.group-label {
  margin: 4px 10px 2px;
  font: var(--rb-ui-label-size, 11px) ui-sans-serif, system-ui, sans-serif;
  letter-spacing: 0.02em;
  color: var(--rb-secondary-text, #707070);
}

.separator {
  height: 1px;
  margin: 4px 6px;
  background: var(--rb-panel-outline, #606060);
  opacity: 0.5;
}
</style>
