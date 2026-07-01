import { createApp } from 'vue';
import App from './App.vue';
import router from './router';
import { initTheme } from './composables/useTheme';
import { initLayout } from './composables/useLayout';
import './styles/base.css';
import './styles/theme.css';

initTheme();
initLayout();

createApp(App).use(router).mount('#app');
