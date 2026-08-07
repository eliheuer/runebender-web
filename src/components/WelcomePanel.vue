<script setup lang="ts">
// Empty-state panel shown when no font is loaded. Mirrors
// runebender-xilem's `views/welcome.rs`: compact upper-left welcome
// UI layered over an interactive demo glyph canvas.

defineProps<{
  // Workspace remembered from a previous visit (File System Access
  // host); shows a one-click reopen above the picker button.
  reopenName?: string | null;
}>();

defineEmits<{
  (e: "openUfo"): void;
  (e: "reopen"): void;
}>();
</script>

<template>
  <div class="welcome">
    <div class="title">Runebender</div>
    <div class="subtitle">A font editor</div>
    <div class="prompt">
      Open the directory holding your <code>.designspace</code> and
      <code>.ufo</code> sources, or drop it here.
    </div>
    <button v-if="reopenName" type="button" class="reopen" @click="$emit('reopen')">
      Reopen {{ reopenName }}
    </button>
    <button type="button" @click="$emit('openUfo')">Open Font Directory...</button>
  </div>
</template>

<style scoped>
.welcome {
  position: absolute;
  left: 16px;
  top: 16px;
  width: 220px;
  height: 200px;
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: flex-start;
  gap: 8px;
  text-align: left;
  color: var(--rb-secondary-text);
}

.title {
  font: 48px ui-sans-serif, system-ui, sans-serif;
  color: var(--rb-primary-text);
  line-height: 1;
}
.subtitle {
  font: var(--rb-ui-font-size) ui-sans-serif, system-ui, sans-serif;
  color: var(--rb-panel-outline);
}
.prompt {
  font: var(--rb-ui-font-size) ui-sans-serif, system-ui, sans-serif;
  color: var(--rb-primary-text);
  line-height: 1.4;
}

button {
  width: 200px;
  height: 32px;
  margin-top: 8px;
  color: var(--rb-primary-text);
  background: var(--rb-panel-background);
  border: var(--rb-stroke-width) solid var(--rb-panel-outline);
  border-radius: var(--rb-button-radius);
  font: var(--rb-ui-font-size) ui-sans-serif, system-ui, sans-serif;
  text-align: center;
  cursor: pointer;
}
button:hover {
  color: var(--rb-accent);
  border-color: var(--rb-accent);
}
button.reopen {
  color: var(--rb-accent);
  margin-top: 0;
}
code {
  font: var(--rb-ui-font-size) ui-monospace, monospace;
  color: var(--rb-accent);
  background: color-mix(in srgb, var(--rb-panel-background) 70%, transparent);
  padding: 1px 5px;
  border-radius: 3px;
}
</style>
