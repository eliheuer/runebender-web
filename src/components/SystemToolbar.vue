<script setup lang="ts">
import GeneratedIcon from "./GeneratedIcon.vue";

// File-operation toolbar peer for runebender-xilem's
// `components/system_toolbar.rs`. Save is wired through the host
// and stays disabled only when there is no loaded font/workspace.
// Close is shown only when the host provides somewhere to close to
// (i.e. running embedded inside another app like ComfyUI).

withDefaults(
  defineProps<{
    saveEnabled?: boolean;
    saveAsEnabled?: boolean;
    closeEnabled?: boolean;
  }>(),
  {
    saveEnabled: false,
    saveAsEnabled: false,
    closeEnabled: false,
  },
);

const emit = defineEmits<{
  (e: "save"): void;
  (e: "saveAs"): void;
  (e: "close"): void;
}>();

function onSave(enabled: boolean) {
  if (!enabled) return;
  emit("save");
}

function onSaveAs(enabled: boolean) {
  if (!enabled) return;
  emit("saveAs");
}
</script>

<template>
  <div class="system-toolbar" role="toolbar" aria-label="System toolbar">
    <button
      type="button"
      class="system-btn"
      :class="{ disabled: !saveEnabled }"
      :disabled="!saveEnabled"
      :aria-disabled="!saveEnabled"
      :title="saveEnabled ? 'Save' : 'Save unavailable'"
      aria-label="Save"
      @click="onSave(saveEnabled)"
    >
      <GeneratedIcon name="save" />
    </button>
    <button
      type="button"
      class="system-btn"
      :class="{ disabled: !saveAsEnabled }"
      :disabled="!saveAsEnabled"
      :aria-disabled="!saveAsEnabled"
      :title="saveAsEnabled ? 'Save As...' : 'Save As unavailable'"
      aria-label="Save As"
      @click="onSaveAs(saveAsEnabled)"
    >
      <GeneratedIcon name="save-as" />
    </button>
    <button
      v-if="closeEnabled"
      type="button"
      class="system-btn close-btn"
      title="Close editor"
      aria-label="Close editor"
      @click="emit('close')"
    >
      <GeneratedIcon name="close" />
    </button>
  </div>
</template>

<style scoped>
.system-toolbar {
  box-sizing: border-box;
  padding: 6px;
  background: var(--rb-panel-background);
  border: var(--rb-stroke-width) solid var(--rb-panel-outline);
  border-radius: var(--rb-panel-radius);
  display: flex;
  align-items: center;
  gap: 6px;
}

.system-btn {
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
}

.system-btn:not(:disabled):hover {
  color: var(--rb-accent);
}

.close-btn:not(:disabled):hover {
  color: var(--rb-warning);
  border-color: var(--rb-warning);
}

.system-btn.disabled {
  cursor: default;
  opacity: 0.55;
}

</style>
