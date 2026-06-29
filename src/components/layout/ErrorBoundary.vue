<template>
  <slot v-if="!err" />
  <div v-else class="err-boundary">
    <div class="err-card">
      <div class="err-icon">⚠</div>
      <div class="err-title">该工具渲染失败</div>
      <div class="err-msg">{{ msg }}</div>
      <button class="err-reload" @click="reload">重新加载</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onErrorCaptured } from 'vue';

const err = ref(false);
const msg = ref('');

onErrorCaptured((e) => {
  err.value = true;
  msg.value = e instanceof Error ? e.message : String(e);
  // 阻止错误继续向上冒泡导致整页白屏
  return false;
});

function reload() {
  err.value = false;
  msg.value = '';
}
</script>

<style scoped>
.err-boundary {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px;
}
.err-card {
  max-width: 420px;
  padding: 28px;
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  text-align: center;
}
.err-icon {
  font-size: 40px;
  color: var(--xuya-warn);
  margin-bottom: 12px;
}
.err-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--xuya-text);
  margin-bottom: 8px;
}
.err-msg {
  font-size: 12.5px;
  color: var(--xuya-text-tertiary);
  font-family: var(--xuya-font-mono);
  margin-bottom: 18px;
  word-break: break-all;
}
.err-reload {
  padding: 7px 18px;
  font-size: 12.5px;
  border: 1px solid var(--xuya-border-strong);
  border-radius: var(--xuya-radius-sm);
  background: var(--xuya-bg-elevated);
  color: var(--xuya-text);
  cursor: pointer;
  transition:
    background var(--xuya-duration-fast) var(--xuya-ease),
    border-color var(--xuya-duration-fast) var(--xuya-ease);
}
.err-reload:hover {
  background: var(--xuya-input-bg);
  border-color: var(--xuya-accent);
  color: var(--xuya-accent);
}
</style>
