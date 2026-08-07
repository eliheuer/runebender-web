<script setup lang="ts">
// What sits behind the glyph you are drawing.
//
// Two separate things, deliberately kept apart: the glyph's own
// background layer (stored in the UFO as public.background, the same
// layer Glyphs and FontLab call the background/mask), and any other
// glyph in the font shown as a reference. The background is an outline
// on the canvas, the reference is a ghost fill, so the two never read
// as the same thing.
const props = defineProps<{
  /** Draw the background layer behind the glyph. */
  show: boolean;
  /** This glyph has something in the background layer. */
  hasBackground: boolean;
  /** Glyph shown behind, or "" for none. */
  reference: string;
  /** Whether the typed reference names a glyph in the font. */
  referenceKnown: boolean;
}>();

const emit = defineEmits<{
  (e: "update:show", v: boolean): void;
  (e: "update:reference", v: string): void;
  (e: "send"): void;
  (e: "swap"): void;
  (e: "clear"): void;
}>();
</script>

<template>
  <section class="layers-panel">
    <div class="label title">Background</div>
    <button
      class="row-btn"
      :class="{ on: props.show }"
      title="Draw this glyph's background layer behind the outline"
      @click="emit('update:show', !props.show)"
    >
      show background
    </button>
    <button
      class="row-btn"
      title="Copy the glyph as it stands into its background layer"
      @click="emit('send')"
    >
      send to background
    </button>
    <button
      class="row-btn"
      :disabled="!props.hasBackground"
      title="Trade the drawing and its background"
      @click="emit('swap')"
    >
      swap with background
    </button>
    <button
      class="row-btn"
      :disabled="!props.hasBackground"
      title="Empty this glyph's background layer"
      @click="emit('clear')"
    >
      clear background
    </button>

    <div class="label">Reference glyph</div>
    <input
      class="field"
      :class="{ unknown: props.reference !== '' && !props.referenceKnown }"
      :value="props.reference"
      placeholder="glyph name"
      spellcheck="false"
      title="Show another glyph behind this one, as a ghost"
      @change="emit('update:reference', ($event.target as HTMLInputElement).value.trim())"
    />
    <button
      v-if="props.reference"
      class="row-btn small"
      @click="emit('update:reference', '')"
    >
      clear reference
    </button>
  </section>
</template>

<style scoped>
.layers-panel {
  width: 138px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 8px;
  background: var(--rb-panel-bg);
  border: 1px solid var(--rb-panel-border);
  border-radius: 8px;
  pointer-events: auto;
}
.label {
  font-size: var(--rb-ui-label-size);
  letter-spacing: 0.02em;
  opacity: 0.5;
  margin-top: 2px;
}
.label.title {
  margin-top: 0;
  opacity: 0.7;
}
.row-btn {
  font: inherit;
  white-space: nowrap;
  font-size: var(--rb-ui-label-size);
  color: inherit;
  background: var(--rb-button-background);
  border: 1px solid var(--rb-panel-outline);
  border-radius: 6px;
  padding: 6px 8px;
  cursor: pointer;
  text-align: left;
}
.row-btn:disabled {
  opacity: 0.35;
  cursor: default;
}
.row-btn.small {
  padding: 4px 8px;
  opacity: 0.8;
  text-align: center;
}
.row-btn.on {
  /* Same "on" look as every other toggle in the editor: accent text and
     border, no fill. */
  color: var(--rb-accent);
  border-color: var(--rb-accent);
}
.field {
  font: inherit;
  font-size: var(--rb-ui-label-size);
  color: inherit;
  /* Typed-into fields sit darker than the panel; buttons sit lighter. */
  background: var(--rb-field-background);
  border: 1px solid var(--rb-panel-outline);
  border-radius: 6px;
  padding: 6px 8px;
  min-width: 0;
}
.field.unknown {
  /* No such glyph — say so quietly rather than clearing what was typed. */
  border-color: var(--rb-warning);
}
</style>
