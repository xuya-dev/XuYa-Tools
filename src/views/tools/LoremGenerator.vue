<template>
  <ToolShell title="占位文本生成" :icon="ScanText" description="生成 Lorem Ipsum / 中文占位文本,段落 / 句子 / 单词。">
    <div class="lorem-controls">
      <div class="ctrl-group">
        <label class="ctrl-label">语言</label>
        <div class="seg">
          <button :class="{ active: lang === 'en' }" @click="lang = 'en'">Lorem Ipsum</button>
          <button :class="{ active: lang === 'zh' }" @click="lang = 'zh'">中文假文</button>
        </div>
      </div>
      <div class="ctrl-group">
        <label class="ctrl-label">类型</label>
        <div class="seg">
          <button v-for="u in UNITS" :key="u.v" :class="{ active: unit === u.v }" @click="unit = u.v">{{ u.label }}</button>
        </div>
      </div>
      <div class="ctrl-group count-group">
        <label class="ctrl-label">数量 {{ count }}</label>
        <input v-model.number="count" type="range" min="1" max="20" class="range" />
      </div>
      <label class="check"><input v-model="startClassic" type="checkbox" :disabled="lang === 'zh'" /> 以 "Lorem ipsum..." 开头</label>
    </div>

    <div class="lorem-result">
      <div class="col-head">
        <span>生成结果 ({{ resultStats }})</span>
        <BaseButton variant="ghost" @click="regenerate"><RefreshCw :size="13" /> 重新生成</BaseButton>
      </div>
      <div class="result-box" @click="copyResult">{{ result }}</div>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { ScanText, RefreshCw } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { copyToClipboard } from '@/composables/useClipboard';

const UNITS = [
  { v: 'paragraphs', label: '段落' },
  { v: 'sentences', label: '句子' },
  { v: 'words', label: '单词' },
] as const;

const EN_WORDS = 'lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod tempor incididunt ut labore et dolore magna aliqua enim ad minim veniam quis nostrud exercitation ullamco laboris nisi aliquip ex ea commodo consequat duis aute irure in reprehenderit voluptate velit esse cillum eu fugiat nulla pariatur excepteur sint occaecat cupidatat non proident sunt culpa qui officia deserunt mollit anim id est laborum'.split(' ');
const ZH_CHARS = '的一是在不了有和人这中大为上个国我以要他时来用们生到作地于出就分对成会可主发年动同工也能下过子说产种面而方后多定行学法所民得经十三之进着等部度家电力里如水化高自二理起小物现实加量都两体制机当使点从业本去把性好应开它合还因由其些然前外天政四日那社义事平形相全表间样与关各重新线内数正心反你明看原又么利比或但质气第向道命此变条只没结解问意建月公无系军很情者最立代想已通并提直题党程展五果料象员革位入常文总次品式活设及管特件长求老头基资边流路级少图山统接知较将组见计别她手角期根论运农指几九区强放决西被干做必战先回则任取据处队南给色光门即保治北造百规热领七海口东导器压志世金增争济阶油思术极交受联什认六共权收证改清美再采转更单风切打白教速花带安场身车例真务具万每目至达走积示议声报斗完类八离华名确才科张信马节话米整空元况今集温传土许步群广石记需段研界拉林律叫且究观越织装影算低持音众书布复容儿须际商非验连断深难近矿千周委素技备半办青省列习响约支般史感劳便团往酸历市克何除消构府称太准精值号率族维划选标写存候毛亲快效斯院查江型眼王按格养易置派层片始却专状育厂京识适属圆包火住调满县局照参红细引听该铁价严龙飞'.split('');

const lang = ref<'en' | 'zh'>('en');
const unit = ref<'paragraphs' | 'sentences' | 'words'>('paragraphs');
const count = ref(3);
const startClassic = ref(true);
const seed = ref(0);

const result = computed(() => {
  void seed.value; // 触发重新生成
  if (lang.value === 'en') return genEn();
  return genZh();
});

function rand(max: number) { return Math.floor(Math.random() * max); }
function pick<T>(arr: T[]): T { return arr[rand(arr.length)]!; }

function genEn(): string {
  if (unit.value === 'words') {
    const ws: string[] = [];
    if (startClassic.value) ws.push('lorem', 'ipsum');
    while (ws.length < count.value * 8) ws.push(pick(EN_WORDS));
    return ws.slice(0, count.value * 8).join(' ');
  }
  if (unit.value === 'sentences') {
    const out: string[] = [];
    for (let i = 0; i < count.value; i++) {
      const len = 8 + rand(8);
      const ws: string[] = [];
      const isFirst = i === 0 && startClassic.value;
      if (isFirst) ws.push('lorem', 'ipsum');
      while (ws.length < len) ws.push(pick(EN_WORDS));
      let s = ws.slice(0, len).join(' ');
      s = s.charAt(0).toUpperCase() + s.slice(1) + '.';
      out.push(s);
    }
    return out.join(' ');
  }
  // paragraphs
  const paras: string[] = [];
  for (let p = 0; p < count.value; p++) {
    const sCount = 3 + rand(3);
    const sents: string[] = [];
    for (let i = 0; i < sCount; i++) {
      const len = 8 + rand(10);
      const ws: string[] = [];
      const isFirst = p === 0 && i === 0 && startClassic.value;
      if (isFirst) ws.push('lorem', 'ipsum');
      while (ws.length < len) ws.push(pick(EN_WORDS));
      let s = ws.slice(0, len).join(' ');
      s = s.charAt(0).toUpperCase() + s.slice(1) + '.';
      sents.push(s);
    }
    paras.push(sents.join(' '));
  }
  return paras.join('\n\n');
}

function genZh(): string {
  if (unit.value === 'words') return Array.from({ length: count.value * 4 }, () => pick(ZH_CHARS)).join('');
  if (unit.value === 'sentences') {
    return Array.from({ length: count.value }, () => {
      const len = 12 + rand(18);
      return Array.from({ length: len }, () => pick(ZH_CHARS)).join('') + '。';
    }).join('');
  }
  return Array.from({ length: count.value }, () => {
    const sCount = 2 + rand(3);
    return Array.from({ length: sCount }, () => {
      const len = 12 + rand(18);
      return Array.from({ length: len }, () => pick(ZH_CHARS)).join('') + '。';
    }).join('');
  }).join('\n\n');
}

const resultStats = computed(() => {
  const s = result.value;
  return `${s.length} 字符 · ${s.split('\n').length} 行`;
});

function regenerate() { seed.value++; }
async function copyResult() {
  if (result.value) await copyToClipboard(result.value, '已复制生成文本');
}
</script>

<style scoped>
.lorem-controls { display: flex; flex-wrap: wrap; gap: 16px; align-items: flex-end; margin-bottom: 20px; }
.ctrl-group { display: flex; flex-direction: column; gap: 6px; }
.ctrl-label { font-size: 11px; font-weight: 600; color: var(--xuya-text-secondary); }
.count-group { min-width: 140px; }
.seg { display: inline-flex; background: var(--xuya-input-bg); border-radius: var(--xuya-radius-sm); padding: 2px; gap: 2px; }
.seg button { padding: 5px 12px; font-size: 12px; color: var(--xuya-text-secondary); background: transparent; border: none; border-radius: 4px; transition: .1s; }
.seg button.active { background: var(--xuya-bg-elevated); color: var(--xuya-accent); font-weight: 600; }
.range { width: 100%; accent-color: var(--xuya-accent); }
.check { display: inline-flex; align-items: center; gap: 6px; font-size: 12px; color: var(--xuya-text-secondary); cursor: pointer; }
.check input { accent-color: var(--xuya-accent); }
.check input:disabled { opacity: .4; }

.lorem-result { display: flex; flex-direction: column; gap: 6px; flex: 1; min-height: 0; }
.col-head { display: flex; justify-content: space-between; align-items: center; font-size: 12px; font-weight: 600; color: var(--xuya-text-secondary); }
.result-box { flex: 1; min-height: 200px; padding: 16px; border-radius: var(--xuya-radius); border: 1px solid var(--xuya-border); background: var(--xuya-input-bg); color: var(--xuya-text); font-size: 13px; line-height: 1.8; overflow: auto; white-space: pre-wrap; cursor: pointer; transition: border-color .12s; }
.result-box:hover { border-color: var(--xuya-accent); }
</style>
