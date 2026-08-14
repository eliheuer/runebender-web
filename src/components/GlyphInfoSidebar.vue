<script setup lang="ts">
// Right-side info panel. Mirrors runebender-xilem's
// `components/glyph_info_panel.rs` field-for-field:
//   Master           — current master name
//   Glyph Name       — UFO glyph identifier
//   Width            — advance width in design units
//   Kerning Groups   — Left / Right from groups.plist
//   Unicode          — first codepoint as 4-digit hex
//   Contours         — number of contours in the active glyph
//
// Label rows are green; values are gray. Empty kerning groups show
// "(empty)" and empty glyph fields show "No Selection", matching xilem.

defineProps<{
  master: string;
  /** Empty when no glyph is selected. */
  name: string;
  /** Uppercase hex, no "U+" prefix. */
  unicode?: string;
  /** Design units. -1 means "no glyph open" (sidebar shows em-dash). */
  width?: number;
  contours?: number;
  /** Full UFO kerning group names. Left is kern2; right is kern1. */
  leftGroup?: string;
  rightGroup?: string;
}>();

function displayGroup(group: string | undefined, prefix: string): string {
  return group ? group.replace(prefix, "") : "(empty)";
}
</script>

<template>
  <aside class="info-sidebar">
    <div class="row">
      <div class="label">Master</div>
      <div class="value">{{ master || "(single UFO)" }}</div>
    </div>

    <div class="row">
      <div class="label">Glyph Name</div>
      <div class="value">{{ name || "No Selection" }}</div>
    </div>

    <div class="row">
      <div class="label">Width</div>
      <div class="value mono">
        {{ width !== undefined && width >= 0 ? Math.round(width) : "—" }}
      </div>
    </div>

    <div class="row group">
      <div class="label">Kerning Groups</div>
      <div class="kerning">
        <div class="kerning-row">
          <span class="kerning-side">Left</span>
          <span class="kerning-val">
            {{ displayGroup(leftGroup, "public.kern2.") }}
          </span>
        </div>
        <div class="kerning-row">
          <span class="kerning-side">Right</span>
          <span class="kerning-val">
            {{ displayGroup(rightGroup, "public.kern1.") }}
          </span>
        </div>
      </div>
    </div>

    <div class="row">
      <div class="label">Unicode</div>
      <div class="value mono">{{ unicode || "No Selection" }}</div>
    </div>

    <div class="row">
      <div class="label">Contours</div>
      <div class="value mono">
        {{ contours !== undefined ? contours : "—" }}
      </div>
    </div>
  </aside>
</template>

<style scoped>
/*
 * Colors from xilem/src/theme.rs:
 * Colour comes from themes/runebender.theme.json via the --rb-*
 * custom properties on the host element. Nothing here names a colour.
 *
 * Width matches xilem's GLYPH_INFO_PANEL_WIDTH (220px).
 */

.info-sidebar {
  width: 220px;
  flex-shrink: 0;
  background: var(--rb-panel-background);
  border: var(--rb-stroke-width) solid var(--rb-panel-outline);
  border-radius: var(--rb-panel-radius);
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 14px;
  overflow-y: auto;
}

.row {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}
.row.group .kerning {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.label {
  color: var(--rb-accent);
  font: var(--rb-ui-font-size) ui-sans-serif, system-ui, sans-serif;
}
.value {
  color: var(--rb-primary-text);
  font: var(--rb-ui-font-size) ui-sans-serif, system-ui, sans-serif;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.value.mono {
  font: var(--rb-ui-font-size) ui-sans-serif, system-ui, sans-serif;
}

.kerning-row {
  display: flex;
  justify-content: space-between;
  gap: 12px;
}
.kerning-side {
  color: var(--rb-secondary-text);
  font: var(--rb-ui-font-size) ui-sans-serif, system-ui, sans-serif;
}
.kerning-val {
  color: var(--rb-primary-text);
  font: var(--rb-ui-font-size) ui-sans-serif, system-ui, sans-serif;
}
</style>
