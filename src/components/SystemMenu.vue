<script setup lang="ts">
// The system-menu button in the upper-left corner: the Runebender rune
// (the app's only raster icon) with a perspective hover tilt and a
// spin on click. The menu itself is SystemMenuPanel.vue, rendered by
// the parent as a bento tile so it joins the panel grid instead of
// floating over it.

import { ref } from "vue";
import menuIcon from "../assets/rb-menu.png";

defineProps<{
  /** Whether the menu panel is showing (drives aria-expanded). */
  open?: boolean;
}>();

const emit = defineEmits<{
  (e: "toggle"): void;
}>();

// One full spin of the rune on every click — restarted via class
// toggle, cleared on animationend.
const spinning = ref(false);

function onMenuButtonClick() {
  spinning.value = true;
  emit("toggle");
}
</script>

<template>
  <div class="system-menu">
    <button
      type="button"
      class="menu-btn"
      title="Menu"
      aria-label="Menu"
      aria-haspopup="menu"
      :aria-expanded="!!open"
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
  /* No button chrome: the rune sits directly on the panel background;
     the hover tilt is the click affordance. */
  appearance: none;
  width: 48px;
  height: 48px;
  box-sizing: border-box;
  padding: 3px;
  background: transparent;
  border: none;
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
  /* The rune's ink sits up and left of the PNG's centre, so nudge the
     image to make it read as centred in the button. */
  transform: translate(2px, 2px);
}
.menu-btn:hover img {
  transform: translate(2px, 2px) rotateY(-16deg) rotateX(7deg) scale(1.08);
  filter: drop-shadow(0 4px 6px rgba(0, 0, 0, 0.7));
}
.menu-btn img.spinning {
  animation: rune-spin 620ms cubic-bezier(0.3, 0.1, 0.25, 1);
}
@keyframes rune-spin {
  from {
    transform: translate(2px, 2px) rotateY(0deg);
  }
  to {
    transform: translate(2px, 2px) rotateY(360deg);
  }
}
</style>
