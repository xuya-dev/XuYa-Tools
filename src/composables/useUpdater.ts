import { ref } from 'vue';
import { isTauri } from '@tauri-apps/api/core';
import { check, type Update } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { getVersion } from '@tauri-apps/api/app';
import { useToast } from './useToast';

/**
 * 应用自动更新 composable。
 *
 * 两个入口:
 *  - checkForUpdateOnStartup():启动时静默检查,发现新版弹 toast 提示用户去设置页更新。
 *  - useUpdater().checkForUpdate(false):设置页手动检查,弹 toast 反馈结果。
 *
 * 设计要点:
 *  - Update 对象(含 downloadAndInstall 方法)不放进 ref,用模块级变量持有,避免响应式包装破坏其内部状态。
 *  - 所有 Tauri 调用都包 try/catch,非 Tauri 环境(纯浏览器,如 vite dev)静默降级。
 *    isTauri() 是同步布尔值(由 Tauri runtime 注入),非 Tauri 环境为 false,无需动态 import。
 */

/** 检查到的更新信息(供 UI 展示)。 */
export interface UpdateInfo {
  version: string;
  date?: string;
  body?: string;
}

// 模块级单例状态(跨组件共享)
const checking = ref(false);
const updateInfo = ref<UpdateInfo | null>(null);
const downloading = ref(false);
const progress = ref(0); // 0~100
const installing = ref(false);
/** 持有当前可用的 Update 对象(不响应式,仅供 downloadAndInstall 调用)。 */
let pendingUpdate: Update | null = null;

/**
 * 检查更新。
 * @param silent 静默模式(启动检查用):无新版/失败不弹 toast,仅发现新版时提示。
 *               非静默(设置页手动检查):任何结果都弹 toast 反馈。
 */
async function checkForUpdate(silent = false): Promise<void> {
  const toast = useToast();
  if (checking.value || downloading.value || installing.value) return;
  if (!isTauri()) return;
  checking.value = true;
  try {
    const update = await check();
    if (update) {
      pendingUpdate = update;
      updateInfo.value = {
        version: update.version,
        date: update.date,
        body: update.body,
      };
      toast.info(`发现新版本 v${update.version},前往「设置 → 关于」更新`, 5000);
    } else {
      pendingUpdate = null;
      updateInfo.value = null;
      if (!silent) toast.success('当前已是最新版本');
    }
  } catch (e) {
    if (!silent) toast.error(`检查更新失败:${formatErr(e)}`);
    // 静默模式下失败不提示,避免打扰
  } finally {
    checking.value = false;
  }
}

/**
 * 下载并安装更新,完成后重启应用。
 * 调用前应已通过 checkForUpdate 发现新版本(pendingUpdate 非空)。
 */
async function downloadAndInstall(): Promise<void> {
  const toast = useToast();
  const update = pendingUpdate;
  if (!update) {
    toast.error('没有可用的更新');
    return;
  }
  if (downloading.value || installing.value) return;
  downloading.value = true;
  progress.value = 0;
  let total = 0;
  let downloaded = 0;
  try {
    await update.downloadAndInstall((event) => {
      if (event.event === 'Started' && event.data.contentLength) {
        total = event.data.contentLength;
      } else if (event.event === 'Progress') {
        downloaded += event.data.chunkLength;
        progress.value = total > 0 ? Math.min(100, Math.round((downloaded / total) * 100)) : 0;
      }
    });
    downloading.value = false;
    progress.value = 100;
    installing.value = true;
    toast.success('下载完成,即将重启安装…');
    await relaunch();
  } catch (e) {
    toast.error(`下载更新失败:${formatErr(e)}`);
    downloading.value = false;
    progress.value = 0;
  }
}

/** 清除当前发现的更新状态(用户忽略或检查后无新版)。 */
function dismissUpdate() {
  pendingUpdate = null;
  updateInfo.value = null;
}

/** 获取应用版本号(读 tauri.conf.json 的 version)。非 Tauri 环境返回空串。 */
async function fetchAppVersion(): Promise<string> {
  if (!isTauri()) return '';
  try {
    return await getVersion();
  } catch {
    return '';
  }
}

function formatErr(e: unknown): string {
  if (e instanceof Error) return e.message;
  return String(e);
}

export function useUpdater() {
  return {
    checking,
    updateInfo,
    downloading,
    progress,
    installing,
    checkForUpdate,
    downloadAndInstall,
    dismissUpdate,
    fetchAppVersion,
  };
}

/**
 * 启动时静默检查更新。在 main.ts mount 之后异步调用,不阻塞渲染。
 * 延迟 3s,避免与启动初始化(主题/布局/代理恢复)抢资源。
 */
export function checkForUpdateOnStartup(): void {
  setTimeout(() => {
    void checkForUpdate(true);
  }, 3000);
}
