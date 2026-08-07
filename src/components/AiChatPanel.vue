<script setup lang="ts">
// PLACEHOLDER — the assistant tab holds its spot in the sidebar and
// does nothing else yet.
//
// An earlier version of this file called the Anthropic API straight
// from the browser with a key the user pasted in (see git history for
// the working version). That works, but asking people to paste an API
// key into a website is not something to release. The direction is a local
// model over an OpenAI-compatible endpoint — Ollama, LM Studio — which
// needs no key, no server of ours, and sends nothing off the machine.
</script>

<template>
  <div class="ai-chat">
    <div class="mark">Coming Soon</div>
    <p class="copy">
      An assistant that can see the open glyph, its outlines and the
      designspace, and answer questions about them.
    </p>
    <p class="copy">
      It will run against a local model, so nothing about your font
      leaves your machine.
    </p>
  </div>
</template>

<style scoped>
.ai-chat {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
  overflow-y: auto;
  scrollbar-width: none;
}
.ai-chat::-webkit-scrollbar {
  display: none;
}

.mark {
  flex: 0 0 auto;
  align-self: flex-start;
  padding: 4px 8px;
  border: 1px dashed var(--rb-panel-outline);
  border-radius: var(--rb-button-radius);
  color: var(--rb-secondary-text);
  font: var(--rb-ui-label-size) ui-sans-serif, system-ui, sans-serif;
  letter-spacing: 0.02em;
}

.copy {
  margin: 0;
  color: var(--rb-secondary-text);
  font: var(--rb-ui-label-size) ui-sans-serif, system-ui, sans-serif;
  line-height: 1.5;
}
</style>
