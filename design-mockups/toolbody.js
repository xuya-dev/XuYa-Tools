// 工具详情区占位内容 (展示工具被打开后右侧的典型形态)
function toolBodyHTML(tool) {
  if (!tool) return `<div class="empty-state">${svg('command',36)}<p>选择左侧工具查看</p></div>`;
  return `
  <div class="tool-body-view">
    <div class="tb-head">
      <div class="tb-head-left">
        <div class="tool-icon" style="width:42px;height:42px;border-radius:10px">${toolIcon(tool,22)}</div>
        <div>
          <h1 class="tb-title">${tool.name}</h1>
          <p class="tb-desc">${tool.desc}</p>
        </div>
      </div>
      <button class="btn ghost">${svg('star',14)} 收藏</button>
    </div>
    <div class="tb-toolbar">
      <button class="btn primary">美化</button>
      <button class="btn">压缩</button>
      <button class="btn">转义</button>
      <button class="btn">反转义</button>
      <span style="flex:1"></span>
      <button class="btn ghost">${svg('history',14)}</button>
    </div>
    <div class="tb-grid">
      <div class="tb-col">
        <div class="tb-col-head"><span>输入</span><span class="tb-stat">28 字符</span></div>
        <pre class="tb-editor mono">{"name":"XuYa","version":1,"features":["json","base64"]}</pre>
      </div>
      <div class="tb-col">
        <div class="tb-col-head"><span>输出</span><span class="tb-stat ok">已格式化</span></div>
        <pre class="tb-editor mono">${'{\n  "name": "XuYa",\n  "version": 1,\n  "features": [\n    "json",\n    "base64"\n  ]\n}'}</pre>
      </div>
    </div>
  </div>`;
}
