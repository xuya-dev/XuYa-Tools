import { createApp } from 'vue';
import App from './App.vue';
import router from './router';
import { initTheme } from './composables/useTheme';
import { initLayout } from './composables/useLayout';
import { checkForUpdateOnStartup } from './composables/useUpdater';
import './styles/base.css';
import './styles/theme.css';

initTheme();
initLayout();

createApp(App).use(router).mount('#app');

// 启动后静默检查更新(延迟 3s,不阻塞渲染;非 Tauri 环境自动降级)
checkForUpdateOnStartup();
