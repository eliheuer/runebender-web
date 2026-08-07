<script setup lang="ts">
// Compact editor master switcher. Mirrors runebender-xilem's
// `components/master_toolbar.rs`: it appears for multi-master
// designspaces and sits beside the workspace toolbar in editor mode.

defineProps<{
  masters: string[];
  activeMaster: number;
  previews?: Array<string | undefined>;
}>();

const emit = defineEmits<{
  (e: "select-master", index: number): void;
}>();

function labelFor(name: string): string {
  return name.trim().slice(0, 1).toLowerCase() || "?";
}
</script>

<template>
  <div class="master-toolbar" role="toolbar" aria-label="Master toolbar">
    <button
      v-for="(name, index) in masters"
      :key="name"
      type="button"
      class="master-btn"
      :class="{ active: index === activeMaster }"
      :title="name"
      :aria-label="`Switch to ${name}`"
      @click="emit('select-master', index)"
    >
      <span
        v-if="previews?.[index]"
        class="master-preview"
        aria-hidden="true"
        v-html="previews[index]"
      />
      <span v-else>{{ labelFor(name) }}</span>
    </button>
  </div>
</template>

<style scoped>
.master-toolbar {
  box-sizing: border-box;
  padding: 6px;
  background: var(--rb-panel-background);
  border: var(--rb-stroke-width) solid var(--rb-panel-outline);
  border-radius: var(--rb-panel-radius);
  display: flex;
  align-items: center;
  gap: 6px;
  pointer-events: auto;
}

.master-btn {
  appearance: none;
  width: 48px;
  height: 48px;
  box-sizing: border-box;
  padding: 0;
  background: var(--rb-button-background);
  border: var(--rb-stroke-width) solid var(--rb-panel-outline);
  border-radius: var(--rb-button-radius);
  color: var(--rb-glyph-preview);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font: var(--rb-ui-font-size) ui-sans-serif, system-ui, sans-serif;
}

.master-btn:hover {
  color: var(--rb-accent);
}

.master-btn.active {
  color: var(--rb-accent);
  border-color: var(--rb-accent);
}

.master-preview {
  width: 28px;
  height: 28px;
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
