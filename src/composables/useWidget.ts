import { ref } from 'vue';

const widgetVisible = ref(false);

export function useWidget() {
  async function toggleWidget() {
    try {
      const { WebviewWindow } = await import('@tauri-apps/api/webviewWindow');

      const existing = await WebviewWindow.getByLabel('widget');
      if (existing) {
        const visible = await existing.isVisible();
        if (visible) {
          await existing.hide();
          widgetVisible.value = false;
        } else {
          await existing.show();
          await existing.setFocus();
          widgetVisible.value = true;
        }
        return;
      }

      const widget = new WebviewWindow('widget', {
        url: '#/widget',
        title: 'XuYa Widget',
        width: 120,
        height: 120,
        resizable: false,
        decorations: false,
        transparent: true,
        alwaysOnTop: true,
        skipTaskbar: true,
        center: false,
        visible: true,
        shadow: false,
      });

      widget.once('tauri://created', () => {
        widgetVisible.value = true;
      });
      widget.once('tauri://error', (e: unknown) => {
        console.error('Widget creation failed:', e);
      });
    } catch (e) {
      console.error('Widget not available:', e);
    }
  }

  return {
    widgetVisible,
    toggleWidget,
  };
}
