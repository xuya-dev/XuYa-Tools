<template>
  <div class="code-view" :class="{ editable }">
    <textarea
      v-if="editable"
      :value="modelValue"
      class="code-area"
      :placeholder="placeholder"
      spellcheck="false"
      @input="emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
    ></textarea>
    <template v-else>
      <code v-if="modelValue" class="hljs-code" v-html="highlighted"></code>
      <span v-else class="code-ph">{{ placeholder }}</span>
    </template>
  </div>
</template>

<script setup lang="ts">
import { shallowRef, watch } from 'vue';
import hljs from 'highlight.js/lib/core';
import jsonLang from 'highlight.js/lib/languages/json';
import sqlLang from 'highlight.js/lib/languages/sql';

hljs.registerLanguage('json', jsonLang);
hljs.registerLanguage('sql', sqlLang);

const props = withDefaults(
  defineProps<{
    modelValue: string;
    editable?: boolean;
    placeholder?: string;
    language?: string;
  }>(),
  { editable: false, placeholder: '', language: 'json' },
);

const emit = defineEmits<{ 'update:modelValue': [value: string] }>();

const highlighted = shallowRef('');

watch(
  () => [props.modelValue, props.language],
  ([code, lang]) => {
    if (!code) {
      highlighted.value = '';
      return;
    }
    try {
      highlighted.value = hljs.highlight(code as string, { language: lang as string }).value;
    } catch {
      highlighted.value = escapeHtml(code as string);
    }
  },
  { immediate: true },
);

function escapeHtml(s: string) {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}
</script>

<style scoped>
.code-view {
  flex: 1;
  min-height: 260px;
  padding: 12px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
  font-size: 12.5px;
  line-height: 1.65;
  letter-spacing: 0.1px;
  font-feature-settings:
    'calt' 1,
    'liga' 1;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-all;
  tab-size: 2;
  transition:
    border-color var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
}

.code-view.editable {
  padding: 0;
  border: none;
  background: transparent;
  overflow: visible;
}

.code-area {
  width: 100%;
  height: 100%;
  min-height: 260px;
  padding: 12px;
  border-radius: var(--xuya-radius);
  border: 1px solid var(--xuya-border);
  background: var(--xuya-input-bg);
  color: var(--xuya-text);
  font-family: var(--xuya-font-mono);
  font-size: 12.5px;
  line-height: 1.65;
  letter-spacing: 0.1px;
  font-feature-settings:
    'calt' 1,
    'liga' 1;
  resize: none;
  tab-size: 2;
  outline: none;
  transition:
    border-color var(--xuya-duration-fast),
    box-shadow var(--xuya-duration-fast);
}
.code-area::placeholder {
  color: var(--xuya-text-tertiary);
}
.code-area:focus {
  border-color: var(--xuya-accent);
  box-shadow: 0 0 0 3px var(--xuya-accent-ring);
}

.code-ph {
  color: var(--xuya-text-tertiary);
}

.hljs-code {
  display: block;
  color: inherit;
  font-family: inherit;
  font-size: inherit;
  line-height: inherit;
}

.hljs-code :deep(.hljs-attr) {
  color: var(--xuya-syn-key);
  font-weight: 500;
}
.hljs-code :deep(.hljs-string) {
  color: var(--xuya-syn-str);
}
.hljs-code :deep(.hljs-number) {
  color: var(--xuya-syn-num);
}
.hljs-code :deep(.hljs-literal) {
  color: var(--xuya-syn-bool);
  font-weight: 500;
}
.hljs-code :deep(.hljs-keyword) {
  color: var(--xuya-syn-key);
  font-weight: 600;
}
.hljs-code :deep(.hljs-built_in) {
  color: var(--xuya-info);
}
.hljs-code :deep(.hljs-comment) {
  color: var(--xuya-text-tertiary);
  font-style: italic;
}
.hljs-code :deep(.hljs-operator) {
  color: var(--xuya-purple);
}
</style>
