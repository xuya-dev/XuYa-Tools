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
      path: '/tools/encoding',
      name: 'encoding',
      component: () => import('@/views/tools/EncodingTool.vue'),
    },
    {
      path: '/tools/jwt',
      name: 'jwt',
      component: () => import('@/views/tools/JwtDecoder.vue'),
    },
    {
      path: '/tools/timestamp',
      name: 'timestamp',
      component: () => import('@/views/tools/TimestampTool.vue'),
    },
    {
      path: '/tools/hashgen',
      name: 'hashgen',
      component: () => import('@/views/tools/HashGenerator.vue'),
    },
    {
      path: '/tools/cli',
      name: 'cli',
      component: () => import('@/views/tools/CliManager.vue'),
    },
  ],
});

export default router;
