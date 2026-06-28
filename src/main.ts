import { createApp } from 'vue';
import App from './App.vue';
import router from './router';
import { initTheme } from './composables/useTheme';
import './styles/base.css';
import './styles/theme.css';

// 在挂载前初始化主题 (index.html 的内联脚本已做了防闪烁,这里做完整初始化)
initTheme();

createApp(App).use(router).mount('#app');
