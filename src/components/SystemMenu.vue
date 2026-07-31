<script setup lang="ts">
// The one-button system menu in the upper-left corner: the Runebender
// logo opens a dropdown with all load/save/workspace actions. This
// replaces the SystemToolbar save-button pair and is the future home
// for settings.

import { onBeforeUnmount, onMounted, ref } from "vue";
import menuIcon from "../assets/rb-menu.png";

withDefaults(
  defineProps<{
    saveEnabled?: boolean;
    saveAsEnabled?: boolean;
    closeEnabled?: boolean;
    /** Workspace remembered from a previous visit (FS Access host). */
    reopenName?: string | null;
  }>(),
  {
    saveEnabled: false,
    saveAsEnabled: false,
    closeEnabled: false,
    reopenName: null,
  },
);

const emit = defineEmits<{
  (e: "openUfo"): void;
  (e: "reopen"): void;
  (e: "save"): void;
  (e: "saveAs"): void;
  (e: "close"): void;
}>();

const open = ref(false);
const rootEl = ref<HTMLElement | null>(null);
// One full spin of the rune on every click — restarted via class
// toggle, cleared on animationend.
const spinning = ref(false);

function onMenuButtonClick() {
  spinning.value = true;
  open.value = !open.value;
}

function pick(
  action: "openUfo" | "reopen" | "save" | "saveAs" | "close",
  enabled = true,
) {
  if (!enabled) return;
  open.value = false;
  emit(action);
}

function onWindowPointerDown(e: PointerEvent) {
  if (open.value && !rootEl.value?.contains(e.target as Node)) {
    open.value = false;
  }
}

function onWindowKeyDown(e: KeyboardEvent) {
  if (open.value && e.key === "Escape") {
    e.stopPropagation();
    open.value = false;
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
  <div ref="rootEl" class="system-menu">
    <button
      type="button"
      class="menu-btn"
      :class="{ active: open }"
      title="Menu"
      aria-label="Menu"
      aria-haspopup="menu"
      :aria-expanded="open"
      @click="onMenuButtonClick"
    >
      <img
        :src="menuIcon"
        :class="{ spinning }"
        alt=""
        draggable="false"
        @animationend="spinning = false"
      />
    </button>

    <div v-if="open" class="dropdown" role="menu">
      <button type="button" role="menuitem" @click="pick('openUfo')">
        Open UFO...
      </button>
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
        type="button"
        role="menuitem"
        :disabled="!saveAsEnabled"
        @click="pick('saveAs', saveAsEnabled)"
      >
        Save As...
      </button>
      <template v-if="closeEnabled">
        <div class="separator" />
        <button type="button" role="menuitem" @click="pick('close')">
          Close Editor
        </button>
      </template>
    </div>
  </div>
</template>

<style scoped>
.system-menu {
  position: relative;
  box-sizing: border-box;
  padding: 6px;
  background: var(--rb-panel-background, #1c1c1c);
  border: var(--rb-stroke-width, 1px) solid var(--rb-panel-outline, #606060);
  border-radius: var(--rb-panel-radius, 12px);
  display: flex;
  align-items: center;
  flex: 0 0 auto;
}

.menu-btn {
  appearance: none;
  width: 48px;
  height: 48px;
  box-sizing: border-box;
  padding: 3px;
  background: var(--rb-button-background, #181818);
  border: var(--rb-stroke-width, 1px) solid var(--rb-panel-outline, #606060);
  border-radius: var(--rb-button-radius, 8px);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  /* Stage for the rune: the transparent-PNG render reads as an object
     in space, so give it real perspective for the tilt and spin. */
  perspective: 260px;
}
.menu-btn img {
  width: 100%;
  height: 100%;
  display: block;
  transform-style: preserve-3d;
  transition: transform 180ms ease;
  filter: drop-shadow(0 2px 3px rgba(0, 0, 0, 0.6));
  will-change: transform;
}
.menu-btn:hover img {
  transform: rotateY(-16deg) rotateX(7deg) scale(1.08);
  filter: drop-shadow(0 4px 6px rgba(0, 0, 0, 0.7));
}
.menu-btn img.spinning {
  animation: rune-spin 620ms cubic-bezier(0.3, 0.1, 0.25, 1);
}
@keyframes rune-spin {
  from {
    transform: rotateY(0deg);
  }
  to {
    transform: rotateY(360deg);
  }
}
.menu-btn:hover,
.menu-btn.active {
  border-color: var(--rb-accent, #18b86f);
}

.dropdown {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  z-index: 100;
  min-width: 220px;
  box-sizing: border-box;
  padding: 6px;
  background: var(--rb-panel-background, #1c1c1c);
  border: var(--rb-stroke-width, 1px) solid var(--rb-panel-outline, #606060);
  border-radius: var(--rb-panel-radius, 12px);
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.dropdown button {
  appearance: none;
  text-align: left;
  padding: 8px 10px;
  background: transparent;
  border: none;
  border-radius: var(--rb-button-radius, 8px);
  color: var(--rb-primary-text, #909090);
  font: 14px ui-sans-serif, system-ui, sans-serif;
  cursor: pointer;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.dropdown button:not(:disabled):hover {
  color: var(--rb-accent, #18b86f);
  background: var(--rb-button-background, #181818);
}
.dropdown button:disabled {
  opacity: 0.55;
  cursor: default;
}
.dropdown button.accent {
  color: var(--rb-accent, #18b86f);
}

.separator {
  height: 1px;
  margin: 4px 6px;
  background: var(--rb-panel-outline, #606060);
  opacity: 0.5;
}
</style>
