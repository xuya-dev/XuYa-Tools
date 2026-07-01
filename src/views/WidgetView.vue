<template>
  <div class="ball-root" :class="{ expanded }" @click="onToggleExpand">
    <!-- 收缩态: 悬浮球 -->
    <div v-if="!expanded" class="ball" data-tauri-drag-region>
      <svg class="ball-ring" viewBox="0 0 100 100">
        <circle class="ring-bg" cx="50" cy="50" r="44" />
        <circle
          class="ring-cpu"
          cx="50"
          cy="50"
          r="44"
          :stroke-dasharray="cpuDash"
          :style="{ stroke: cpuColor }"
        />
        <circle class="ring-bg-inner" cx="50" cy="50" r="36" />
        <circle
          class="ring-mem"
          cx="50"
          cy="50"
          r="36"
          :stroke-dasharray="memDash"
          :style="{ stroke: memColor }"
        />
      </svg>
      <div class="ball-center">
        <span class="ball-cpu" :style="{ color: cpuColor }">
          {{ (stats?.cpuUsage ?? 0).toFixed(0) }}
        </span>
        <span class="ball-unit">%</span>
      </div>
      <div class="ball-net">
        <span class="bn-down">↓{{ netRxText }}</span>
        <span class="bn-up">↑{{ netTxText }}</span>
      </div>
    </div>

    <!-- 展开态: 详情面板 -->
    <div v-else class="ball-panel" data-tauri-drag-region>
      <div class="bp-head" data-tauri-drag-region>
        <img class="bp-logo" src="/Logo.png" alt="" />
        <span class="bp-title">系统监控</span>
        <div class="bp-drag-hint">⋮⋮ 拖拽</div>
      </div>

      <div class="bp-body">
        <div class="bp-row">
          <div class="bp-label">
            <Cpu :size="12" />
            CPU
          </div>
          <div class="bp-ring-wrap">
            <svg class="bp-ring" viewBox="0 0 60 60">
              <circle class="ring-bg" cx="30" cy="30" r="26" />
              <circle
                class="ring-fill"
                cx="30"
                cy="30"
                r="26"
                :stroke-dasharray="cpuDashSmall"
                :style="{ stroke: cpuColor }"
              />
            </svg>
            <span class="bp-ring-val" :style="{ color: cpuColor }">
              {{ (stats?.cpuUsage ?? 0).toFixed(0) }}%
            </span>
          </div>
          <span class="bp-sub">{{ stats?.cpuCount ?? 0 }} 核</span>
        </div>

        <div class="bp-row">
          <div class="bp-label">
            <MemoryStick :size="12" />
            内存
          </div>
          <div class="bp-ring-wrap">
            <svg class="bp-ring" viewBox="0 0 60 60">
              <circle class="ring-bg" cx="30" cy="30" r="26" />
              <circle
                class="ring-fill"
                cx="30"
                cy="30"
                r="26"
                :stroke-dasharray="memDashSmall"
                :style="{ stroke: memColor }"
              />
            </svg>
            <span class="bp-ring-val" :style="{ color: memColor }">
              {{ (stats?.memPercent ?? 0).toFixed(0) }}%
            </span>
          </div>
          <span class="bp-sub">
            {{ (stats?.memUsedGb ?? 0).toFixed(1) }}/{{ (stats?.memTotalGb ?? 0).toFixed(1) }}G
          </span>
        </div>

        <div class="bp-net">
          <div class="bp-net-row">
            <ArrowDown :size="13" class="ic-down" />
            <span class="bp-net-val">{{ netRxText }}</span>
          </div>
          <div class="bp-net-row">
            <ArrowUp :size="13" class="ic-up" />
            <span class="bp-net-val">{{ netTxText }}</span>
          </div>
        </div>

        <div class="bp-uptime">
          <Clock :size="11" />
          {{ formatUptime(stats?.uptimeSecs ?? 0) }}
        </div>
      </div>

      <div class="bp-foot">
        <button class="bp-btn" @click.stop="closeWidget">关闭</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { Cpu, MemoryStick, ArrowDown, ArrowUp, Clock } from '@lucide/vue';
import { useSystemStats, formatSpeed, formatUptime } from '@/composables/useSystemStats';

const { stats, stop } = useSystemStats(1000);
onUnmounted(() => stop());

const expanded = ref(false);

function onToggleExpand() {
  expanded.value = !expanded.value;
  resizeWindow();
}

async function resizeWindow() {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    const win = getCurrentWindow();
    if (expanded.value) {
      await win.setSize({ width: 220, height: 320 } as never);
    } else {
      await win.setSize({ width: 120, height: 120 } as never);
    }
  } catch {
    /* non-Tauri */
  }
}

const R_CPU = 44;
const R_MEM = 36;
const C_CPU = 2 * Math.PI * R_CPU;
const C_MEM = 2 * Math.PI * R_MEM;

const cpuDash = computed(() => {
  const pct = (stats.value?.cpuUsage ?? 0) / 100;
  return `${(pct * C_CPU).toFixed(1)} ${C_CPU.toFixed(1)}`;
});
const memDash = computed(() => {
  const pct = (stats.value?.memPercent ?? 0) / 100;
  return `${(pct * C_MEM).toFixed(1)} ${C_MEM.toFixed(1)}`;
});
const cpuDashSmall = computed(() => {
  const pct = (stats.value?.cpuUsage ?? 0) / 100;
  const c = 2 * Math.PI * 26;
  return `${(pct * c).toFixed(1)} ${c.toFixed(1)}`;
});
const memDashSmall = computed(() => {
  const pct = (stats.value?.memPercent ?? 0) / 100;
  const c = 2 * Math.PI * 26;
  return `${(pct * c).toFixed(1)} ${c.toFixed(1)}`;
});

const cpuColor = computed(() => {
  const v = stats.value?.cpuUsage ?? 0;
  return v > 80 ? '#f85149' : v > 60 ? '#fbbf24' : '#2dd4bf';
});
const memColor = computed(() => {
  const v = stats.value?.memPercent ?? 0;
  return v > 85 ? '#f85149' : v > 70 ? '#fbbf24' : '#60a5fa';
});

const netRxText = computed(() => formatSpeed(stats.value?.netRxKbps ?? 0));
const netTxText = computed(() => formatSpeed(stats.value?.netTxKbps ?? 0));

async function closeWidget() {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    await getCurrentWindow().hide();
  } catch {
    /* non-Tauri */
  }
}

onMounted(() => resizeWindow());
</script>

<style scoped>
.ball-root {
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}

/* ====== 悬浮球 (收缩态) ====== */
.ball {
  width: 120px;
  height: 120px;
  position: relative;
  cursor: pointer;
  -webkit-app-region: drag;
}
.ball-root.expanded .ball {
  display: none;
}

.ball-ring {
  position: absolute;
  inset: 0;
  width: 120px;
  height: 120px;
  transform: rotate(-90deg);
}
.ring-bg {
  fill: rgba(22, 27, 34, 0.85);
  stroke: rgba(255, 255, 255, 0.08);
  stroke-width: 3;
}
.ring-bg-inner {
  fill: none;
  stroke: rgba(255, 255, 255, 0.05);
  stroke-width: 2;
}
.ring-cpu {
  fill: none;
  stroke-width: 4;
  stroke-linecap: round;
  transition:
    stroke-dasharray 0.5s ease,
    stroke 0.3s;
  filter: drop-shadow(0 0 3px currentColor);
}
.ring-mem {
  fill: none;
  stroke-width: 3;
  stroke-linecap: round;
  transition:
    stroke-dasharray 0.5s ease,
    stroke 0.3s;
  filter: drop-shadow(0 0 2px currentColor);
}

.ball-center {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  -webkit-app-region: no-drag;
}
.ball-cpu {
  font-size: 30px;
  font-weight: 800;
  font-family: var(--xuya-font-mono);
  line-height: 1;
  transition: color 0.3s;
}
.ball-unit {
  font-size: 11px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.4);
  margin-top: -2px;
}

.ball-net {
  position: absolute;
  bottom: 14px;
  left: 0;
  right: 0;
  display: flex;
  justify-content: center;
  gap: 6px;
  font-size: 9px;
  font-family: var(--xuya-font-mono);
  pointer-events: none;
}
.bn-down {
  color: #3fb950;
}
.bn-up {
  color: #58a6ff;
}

/* ====== 展开面板 ====== */
.ball-panel {
  width: 220px;
  height: 320px;
  background: rgba(13, 17, 23, 0.92);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 14px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
}
.ball-root:not(.expanded) .ball-panel {
  display: none;
}

.bp-head {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 9px 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  -webkit-app-region: drag;
}
.bp-logo {
  width: 18px;
  height: 18px;
  border-radius: 4px;
}
.bp-title {
  font-size: 11px;
  font-weight: 700;
  color: #e6edf3;
  flex: 1;
}
.bp-drag-hint {
  font-size: 9px;
  color: rgba(255, 255, 255, 0.2);
  cursor: grab;
}

.bp-body {
  flex: 1;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  -webkit-app-region: no-drag;
}

.bp-row {
  display: flex;
  align-items: center;
  gap: 10px;
}
.bp-label {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 10px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.5);
  width: 36px;
  flex-shrink: 0;
}
.bp-ring-wrap {
  position: relative;
  width: 46px;
  height: 46px;
  flex-shrink: 0;
}
.bp-ring {
  width: 46px;
  height: 46px;
  transform: rotate(-90deg);
}
.bp-ring .ring-bg {
  fill: none;
  stroke: rgba(255, 255, 255, 0.06);
  stroke-width: 4;
}
.bp-ring .ring-fill {
  fill: none;
  stroke-width: 4;
  stroke-linecap: round;
  transition: stroke-dasharray 0.5s;
}
.bp-ring-val {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
  font-weight: 700;
  font-family: var(--xuya-font-mono);
}
.bp-sub {
  font-size: 9px;
  color: rgba(255, 255, 255, 0.35);
  font-family: var(--xuya-font-mono);
}

.bp-net {
  display: flex;
  gap: 16px;
  padding: 6px 0;
  border-top: 1px solid rgba(255, 255, 255, 0.04);
}
.bp-net-row {
  display: flex;
  align-items: center;
  gap: 4px;
}
.ic-down {
  color: #3fb950;
}
.ic-up {
  color: #58a6ff;
}
.bp-net-val {
  font-size: 12px;
  font-weight: 600;
  color: #e6edf3;
  font-family: var(--xuya-font-mono);
}

.bp-uptime {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 10px;
  color: rgba(255, 255, 255, 0.3);
  font-family: var(--xuya-font-mono);
}

.bp-foot {
  padding: 8px 12px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  -webkit-app-region: no-drag;
}
.bp-btn {
  width: 100%;
  padding: 5px;
  font-size: 11px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.5);
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.12s;
}
.bp-btn:hover {
  background: var(--xuya-danger-soft);
  color: var(--xuya-danger);
  border-color: var(--xuya-danger);
}
</style>
