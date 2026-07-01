import { createRouter, createWebHashHistory } from 'vue-router';

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', redirect: '/tools/json' },
    {
      path: '/tools/converter',
      name: 'converter',
      component: () => import('@/views/tools/FormatConverter.vue'),
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
    { path: '/tools/diff', name: 'diff', component: () => import('@/views/tools/TextDiff.vue') },
    {
      path: '/tools/markdown',
      name: 'markdown',
      component: () => import('@/views/tools/MarkdownPreview.vue'),
    },
    { path: '/tools/sql', name: 'sql', component: () => import('@/views/tools/SqlFormatter.vue') },
    {
      path: '/tools/urlparser',
      name: 'urlparser',
      component: () => import('@/views/tools/UrlParser.vue'),
    },
    {
      path: '/tools/baseconv',
      name: 'baseconv',
      component: () => import('@/views/tools/BaseConverter.vue'),
    },
    {
      path: '/tools/encoding',
      name: 'encoding',
      component: () => import('@/views/tools/EncodingTool.vue'),
    },
    {
      path: '/tools/imgtool',
      name: 'imgtool',
      component: () => import('@/views/tools/ImageTool.vue'),
    },
    { path: '/tools/color', name: 'color', component: () => import('@/views/tools/ColorTool.vue') },
    {
      path: '/tools/qrcode',
      name: 'qrcode',
      component: () => import('@/views/tools/QrCodeTool.vue'),
    },
    {
      path: '/tools/qrdecode',
      name: 'qrdecode',
      component: () => import('@/views/tools/QrCodeDecoder.vue'),
    },
    { path: '/tools/jwt', name: 'jwt', component: () => import('@/views/tools/JwtDecoder.vue') },
    {
      path: '/tools/idgen',
      name: 'idgen',
      component: () => import('@/views/tools/IdGenerator.vue'),
    },
    {
      path: '/tools/crypto',
      name: 'crypto',
      component: () => import('@/views/tools/CryptoTool.vue'),
    },
    {
      path: '/tools/keygen',
      name: 'keygen',
      component: () => import('@/views/tools/KeyPairTool.vue'),
    },
    {
      path: '/tools/textproc',
      name: 'textproc',
      component: () => import('@/views/tools/TextProcessor.vue'),
    },
    {
      path: '/tools/timestamp',
      name: 'timestamp',
      component: () => import('@/views/tools/TimestampTool.vue'),
    },
    { path: '/tools/cron', name: 'cron', component: () => import('@/views/tools/CronParser.vue') },
    { path: '/tools/http', name: 'http', component: () => import('@/views/tools/HttpTool.vue') },
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
    { path: '/settings', name: 'settings', component: () => import('@/views/SettingsView.vue') },
    { path: '/tools/cli', name: 'cli', component: () => import('@/views/tools/CliManager.vue') },
    {
      path: '/tools/proxy',
      name: 'proxy',
      component: () => import('@/views/tools/ProxyDashboard.vue'),
    },
  ],
});

const PREFETCH_NAMES = ['sql', 'diff', 'color', 'jwt', 'idgen', 'crypto', 'baseconv', 'http'];
const prefetched = new Set<string>();
router.afterEach((to) => {
  if (!to.name || typeof to.name !== 'string') return;
  for (const n of PREFETCH_NAMES) {
    if (prefetched.has(n)) continue;
    const route = router.getRoutes().find((r) => r.name === n);
    const loader = route?.components?.default;
    if (loader && typeof loader === 'function') {
      prefetched.add(n);
      (loader as () => Promise<unknown>)();
    }
  }
});

export default router;
