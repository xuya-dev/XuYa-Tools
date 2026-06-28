import { createRouter, createWebHashHistory } from 'vue-router';

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    {
      path: '/',
      redirect: '/tools/json',
    },
    {
      path: '/tools/json',
      name: 'json',
      component: () => import('@/views/tools/JsonFormatter.vue'),
    },
    {
      path: '/tools/regex',
      name: 'regex',
      component: () => import('@/views/tools/RegexTester.vue'),
    },
    {
      path: '/tools/diff',
      name: 'diff',
      component: () => import('@/views/tools/TextDiff.vue'),
    },
    {
      path: '/tools/markdown',
      name: 'markdown',
      component: () => import('@/views/tools/MarkdownPreview.vue'),
    },
    {
      path: '/tools/sql',
      name: 'sql',
      component: () => import('@/views/tools/SqlFormatter.vue'),
    },
    {
      path: '/tools/encoding',
      name: 'encoding',
      component: () => import('@/views/tools/EncodingTool.vue'),
    },
    {
      path: '/tools/color',
      name: 'color',
      component: () => import('@/views/tools/ColorTool.vue'),
    },
    {
      path: '/tools/qrcode',
      name: 'qrcode',
      component: () => import('@/views/tools/QrCodeTool.vue'),
    },
    {
      path: '/tools/lorem',
      name: 'lorem',
      component: () => import('@/views/tools/LoremGenerator.vue'),
    },
    {
      path: '/tools/jwt',
      name: 'jwt',
      component: () => import('@/views/tools/JwtDecoder.vue'),
    },
    {
      path: '/tools/hashgen',
      name: 'hashgen',
      component: () => import('@/views/tools/HashGenerator.vue'),
    },
    {
      path: '/tools/timestamp',
      name: 'timestamp',
      component: () => import('@/views/tools/TimestampTool.vue'),
    },
    {
      path: '/tools/cron',
      name: 'cron',
      component: () => import('@/views/tools/CronParser.vue'),
    },
    {
      path: '/tools/http',
      name: 'http',
      component: () => import('@/views/tools/HttpTool.vue'),
    },
    {
      path: '/tools/websocket',
      name: 'websocket',
      component: () => import('@/views/tools/WebSocketTool.vue'),
    },
    {
      path: '/tools/httpstatus',
      name: 'httpstatus',
      component: () => import('@/views/tools/HttpStatusTool.vue'),
    },
    {
      path: '/tools/ipcalc',
      name: 'ipcalc',
      component: () => import('@/views/tools/IpCalculator.vue'),
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/SettingsView.vue'),
    },
    {
      path: '/tools/cli',
      name: 'cli',
      component: () => import('@/views/tools/CliManager.vue'),
    },
    {
      path: '/tools/proxy',
      name: 'proxy',
      component: () => import('@/views/tools/ProxyDashboard.vue'),
    },
  ],
});

export default router;
