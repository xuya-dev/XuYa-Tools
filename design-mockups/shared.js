// ===== 工具数据 (6 个工具 + 分类) =====
const CATEGORIES = {
  converter: '格式转换', encoder: '编码解码', crypto: '加密/生成',
  datetime: '日期时间', devops: '开发配置',
};
const TOOLS = [
  { id:'json', name:'JSON 格式化', desc:'美化、压缩、转义、反转义 JSON,带错误定位与字符统计。', cat:'converter', icon:'braces' },
  { id:'encoding', name:'编码转换', desc:'Base64 / URL / HTML 实体 / Hex / Binary 双向转换。', cat:'encoder', icon:'binary' },
  { id:'jwt', name:'JWT 解析', desc:'解码 JWT 的 Header / Payload / Signature,查看算法与过期时间。', cat:'crypto', icon:'key' },
  { id:'timestamp', name:'时间戳转换', desc:'Unix 时间戳与可读时间互转,秒/毫秒,实时当前时间。', cat:'datetime', icon:'clock' },
  { id:'hashgen', name:'UUID / 密码 / 哈希', desc:'生成 UUID、随机密码,计算 MD5 / SHA-256 等哈希。', cat:'crypto', icon:'fingerprint' },
  { id:'cli', name:'CLI 配置 / 反代', desc:'管理 Claude Code / Codex CLI 配置,本地反代与请求统计。', cat:'devops', icon:'terminal' },
];

// ===== SVG 图标库 (stroke=currentColor, 24x24) =====
const ICONS = {
  braces: '<path d="M8 3H7a2 2 0 0 0-2 2v5a2 2 0 0 1-2 2 2 2 0 0 1 2 2v5a2 2 0 0 0 2 2h1M16 3h1a2 2 0 0 1 2 2v5a2 2 0 0 0 2 2 2 2 0 0 0-2 2v5a2 2 0 0 1-2 2h-1"/>',
  binary: '<rect x="14" y="14" width="4" height="6" rx="2"/><rect x="6" y="14" width="4" height="6" rx="2"/><rect x="6" y="4" width="4" height="6" rx="2"/><path d="M16 4h2M6 14h2M14 20h2M6 20h2M16 14h2M6 4h2"/>',
  key: '<circle cx="7.5" cy="15.5" r="5.5"/><path d="m21 2-9.6 9.6M15.5 7.5l3 3L22 7l-3-3"/>',
  clock: '<circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/>',
  fingerprint: '<path d="M2 12C2 6.5 6.5 2 12 2a10 10 0 0 1 8 4M5 19a14 14 0 0 0 1-7 6 6 0 0 1 12 0c0 2-.5 4-1 6M12 12v3M9 16a8 8 0 0 1 0-4M19.5 16a5 5 0 0 0 .5-2"/>',
  terminal: '<polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/>',
  search: '<circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  chevronRight: '<path d="m9 18 6-6-6-6"/>',
  chevronLeft: '<path d="m15 18-6-6 6-6"/>',
  back: '<path d="M19 12H5M12 19l-7-7 7-7"/>',
  grid: '<rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/>',
  list: '<line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><line x1="3" y1="6" x2="3.01" y2="6"/><line x1="3" y1="12" x2="3.01" y2="12"/><line x1="3" y1="18" x2="3.01" y2="18"/>',
  settings: '<path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/>',
  star: '<polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>',
  command: '<path d="M18 3a3 3 0 0 0-3 3v12a3 3 0 0 0 3 3 3 3 0 0 0 3-3 3 3 0 0 0-3-3H6a3 3 0 0 0-3 3 3 3 0 0 0 3 3 3 3 0 0 0 3-3V6a3 3 0 0 0-3-3 3 3 0 0 0-3 3 3 3 0 0 0 3 3h12a3 3 0 0 0 3-3 3 3 0 0 0-3-3z"/>',
  moon: '<path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/>',
  arrowRight: '<path d="M5 12h14M12 5l7 7-7 7"/>',
  panelLeft: '<rect width="18" height="18" x="3" y="3" rx="2"/><path d="M9 3v18"/>',
  history: '<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/><path d="M12 7v5l4 2"/>',
};
function svg(name, size=20, sw=2) {
  const p = ICONS[name] || ICONS.grid;
  return `<svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="${sw}" stroke-linecap="round" stroke-linejoin="round">${p}</svg>`;
}
function toolIcon(t, size=20) { return svg(t.icon, size); }

// 标题栏 HTML(所有布局顶部一致)
function titlebarHTML() {
  return `
  <div class="titlebar">
    <div class="brand"><div class="logo">⌘</div><span>XuYa Tools</span></div>
    <div class="spacer"></div>
    <button class="win-ctrl-btn">${svg('moon',14)}</button>
    <div class="win-ctrl">
      <button>—</button><button>▢</button><button class="close">✕</button>
    </div>
  </div>`;
}
