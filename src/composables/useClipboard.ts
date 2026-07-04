import { useToast } from './useToast';

/**
 * 复制文本到剪贴板,并给出统一的 Toast 反馈。
 */
export async function copyToClipboard(
  text: string,
  successMsg = '已复制到剪贴板',
): Promise<boolean> {
  const toast = useToast();
  try {
    await navigator.clipboard.writeText(text);
    toast.success(successMsg);
    return true;
  } catch {
    // 降级方案:execCommand
    try {
      const ta = document.createElement('textarea');
      ta.value = text;
      ta.style.position = 'fixed';
      ta.style.opacity = '0';
      document.body.appendChild(ta);
      ta.select();
      document.execCommand('copy');
      document.body.removeChild(ta);
      toast.success(successMsg);
      return true;
    } catch {
      toast.error('复制失败');
      return false;
    }
  }
}
