<script setup lang="ts">
import GeneratedIcon from "./GeneratedIcon.vue";

// Sub-toolbar peer for runebender-xilem's
// `components/text_direction_toolbar.rs`. Appears when Text is active
// and chooses the direction used by active Text entry.

export type TextDirection = "ltr" | "rtl";
/** "auto" lets every line follow its own script. */
export type TextDirectionMode = "auto" | TextDirection;

defineProps<{
  active: TextDirectionMode;
  /** Direction the cursor's line resolves to while in auto mode. */
  resolved?: TextDirection;
}>();

defineEmits<{
  (e: "select", direction: TextDirectionMode): void;
}>();

const directions = [
  ["auto", "Auto (per line, from the script you type)"],
  ["ltr", "Left to Right (whole buffer)"],
  ["rtl", "Right to Left (whole buffer)"],
] as const;

const DIRECTION_ICONS: Record<TextDirection, string> = {
  ltr: "text-ltr",
  rtl: "text-rtl",
};
</script>

<template>
  <div
    class="text-direction-toolbar"
    role="toolbar"
    aria-label="Text direction toolbar"
  >
    <button
      v-for="[id, label] in directions"
      :key="id"
      type="button"
      class="direction-btn"
      :class="{ active: id === active }"
      :title="label"
      :aria-label="label"
      :aria-pressed="id === active"
      @click="$emit('select', id)"
    >
      <!-- Auto shows which way the current line came out. -->
      <span v-if="id === 'auto'" class="auto-label">
        Auto
        <span class="auto-resolved">{{ (resolved ?? "ltr").toUpperCase() }}</span>
      </span>
      <GeneratedIcon v-else :name="DIRECTION_ICONS[id]" />
    </button>
  </div>
</template>

<style scoped>
.text-direction-toolbar {
  background: var(--rb-panel-background, #1c1c1c);
  border: var(--rb-stroke-width, 1px) solid var(--rb-panel-outline, #606060);
  border-radius: var(--rb-panel-radius, 12px);
  padding: 6px;
  display: flex;
  flex-direction: row;
  gap: 6px;
  width: fit-content;
  flex-shrink: 0;
}

.auto-label {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  font: 11px ui-sans-serif, system-ui, sans-serif;
  line-height: 1;
}
.auto-resolved {
  font-size: 9px;
  opacity: 0.75;
  font-variant-numeric: tabular-nums;
}

.direction-btn {
  appearance: none;
  font: inherit;
  width: 48px;
  height: 48px;
  background: var(--rb-button-background, #181818);
  color: var(--rb-glyph-preview, #808080);
  border: var(--rb-stroke-width, 1px) solid var(--rb-panel-outline, #606060);
  border-radius: var(--rb-button-radius, 8px);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
}

.direction-btn:hover,
.direction-btn.active {
  color: var(--rb-accent, #18b86f);
  border-color: var(--rb-accent, #18b86f);
}

</style>
