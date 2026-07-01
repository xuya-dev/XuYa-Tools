import { createRouter, createWebHashHistory } from 'vue-router';

const lazy = (path: string) => () => import(/* @vite-ignore */ path);

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/', redirect: '/tools/json' },
    { path: '/tools/converter', name: 'converter', component: lazy('@/views/tools/FormatConverter.vue') },
    { path: '/tools/json', name: 'json', component: lazy('@/views/tools/JsonFormatter.vue') },
    { path: '/tools/regex', name: 'regex', component: lazy('@/views/tools/RegexTester.vue') },
    { path: '/tools/diff', name: 'diff', component: lazy('@/views/tools/TextDiff.vue') },
    { path: '/tools/markdown', name: 'markdown', component: lazy('@/views/tools/MarkdownPreview.vue') },
    { path: '/tools/sql', name: 'sql', component: lazy('@/views/tools/SqlFormatter.vue') },
    { path: '/tools/urlparser', name: 'urlparser', component: lazy('@/views/tools/UrlParser.vue') },
    { path: '/tools/baseconv', name: 'baseconv', component: lazy('@/views/tools/BaseConverter.vue') },
    { path: '/tools/encoding', name: 'encoding', component: lazy('@/views/tools/EncodingTool.vue') },
    { path: '/tools/imgtool', name: 'imgtool', component: lazy('@/views/tools/ImageTool.vue') },
    { path: '/tools/color', name: 'color', component: lazy('@/views/tools/ColorTool.vue') },
    { path: '/tools/qrcode', name: 'qrcode', component: lazy('@/views/tools/QrCodeTool.vue') },
    { path: '/tools/qrdecode', name: 'qrdecode', component: lazy('@/views/tools/QrCodeDecoder.vue') },
    { path: '/tools/jwt', name: 'jwt', component: lazy('@/views/tools/JwtDecoder.vue') },
    { path: '/tools/idgen', name: 'idgen', component: lazy('@/views/tools/IdGenerator.vue') },
    { path: '/tools/crypto', name: 'crypto', component: lazy('@/views/tools/CryptoTool.vue') },
    { path: '/tools/keygen', name: 'keygen', component: lazy('@/views/tools/KeyPairTool.vue') },
    { path: '/tools/textproc', name: 'textproc', component: lazy('@/views/tools/TextProcessor.vue') },
    { path: '/tools/timestamp', name: 'timestamp', component: lazy('@/views/tools/TimestampTool.vue') },
    { path: '/tools/cron', name: 'cron', component: lazy('@/views/tools/CronParser.vue') },
    { path: '/tools/http', name: 'http', component: lazy('@/views/tools/HttpTool.vue') },
    { path: '/tools/websocket', name: 'websocket', component: lazy('@/views/tools/WebSocketTool.vue') },
    { path: '/tools/httpstatus', name: 'httpstatus', component: lazy('@/views/tools/HttpStatusTool.vue') },
    { path: '/tools/ipcalc', name: 'ipcalc', component: lazy('@/views/tools/IpCalculator.vue') },
    { path: '/settings', name: 'settings', component: lazy('@/views/SettingsView.vue') },
    { path: '/tools/cli', name: 'cli', component: lazy('@/views/tools/CliManager.vue') },
    { path: '/tools/proxy', name: 'proxy', component: lazy('@/views/tools/ProxyDashboard.vue') },
  ],
});

const HIGH_PRIORITY = ['json', 'regex', 'converter', 'encoding', 'timestamp'];
router.afterEach((to) => {
  if (to.name && HIGH_PRIORITY.includes(to.name as string)) {
    const names = ['http', 'sql', 'diff', 'color', 'jwt', 'idgen', 'crypto', 'hashgen', 'baseconv'];
    for (const n of names) {
      const r = router.getRoutes().find((route) => route.name === n);
      if (r?.components?.default && typeof r.components.default === 'function') {
        (r.components.default as () => Promise<unknown>)();
      }
    }
  }
});

export default router;
