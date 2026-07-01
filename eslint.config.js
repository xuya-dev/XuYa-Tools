import js from '@eslint/js';
import tseslint from 'typescript-eslint';
import pluginVue from 'eslint-plugin-vue';
import prettierConfig from 'eslint-config-prettier';

export default tseslint.config(
  // 基础:ESLint 推荐规则
  js.configs.recommended,

  // TypeScript 推荐规则 (类型感知)
  ...tseslint.configs.recommended,

  // Vue 推荐规则
  ...pluginVue.configs['flat/recommended'],

  // 关闭与 Prettier 冲突的格式类规则
  prettierConfig,

  // 项目定制
  {
    languageOptions: {
      parserOptions: {
        parser: tseslint.parser,
        sourceType: 'module',
        ecmaVersion: 'latest',
      },
      globals: {
        // 浏览器 / Tauri 运行时全局
        window: 'readonly',
        document: 'readonly',
        localStorage: 'readonly',
        matchMedia: 'readonly',
        setTimeout: 'readonly',
        clearTimeout: 'readonly',
        console: 'readonly',
        confirm: 'readonly',
        navigator: 'readonly',
        location: 'readonly',
        __TAURI_INTERNALS__: 'readonly',
      },
    },
    rules: {
      // Vue:SFC 中组件名默认按文件名,无需强制多单词
      'vue/multi-word-component-names': 'off',
      // 允许 template 中使用 v-html (Markdown/JSON 工具确实需要)
      'vue/no-v-html': 'off',
      // TS:未使用变量默认告警,允许前缀 _ 跳过
      '@typescript-eslint/no-unused-vars': [
        'warn',
        { argsIgnorePattern: '^_', varsIgnorePattern: '^_' },
      ],
      // 允许使用 any (Tauri IPC 边界、第三方类型不完整时需要)
      '@typescript-eslint/no-explicit-any': 'off',
      // console 在桌面工具里是合理的调试输出
      'no-console': 'off',
    },
  },

  // 忽略路径
  {
    ignores: [
      'dist/**',
      'node_modules/**',
      'src-tauri/target/**',
      'src-tauri/gen/**',
      '**/*.d.ts',
      'scripts/**',
    ],
  },
);
