<template>
  <ToolShell
    title="测试数据生成"
    :icon="Users"
    description="生成身份证、手机号、姓名等测试数据，基于模板生成真实风格证件图片（仅供开发测试）。"
  >
    <div class="gen-toolbar">
      <div class="opt-group">
        <span class="opt-label">数量</span>
        <select v-model.number="count" class="opt-select" @change="generate">
          <option :value="1">1 条</option>
          <option :value="5">5 条</option>
          <option :value="10">10 条</option>
          <option :value="20">20 条</option>
          <option :value="50">50 条</option>
        </select>
      </div>
      <span class="sep"></span>
      <label class="toggle">
        <input v-model="maskId" type="checkbox" />
        身份证脱敏
      </label>
      <label class="toggle">
        <input v-model="maskPhoneToggle" type="checkbox" />
        手机脱敏
      </label>
      <span style="flex: 1"></span>
      <BaseButton variant="primary" @click="generate">
        <RefreshCw :size="14" />
        重新生成
      </BaseButton>
      <BaseButton variant="ghost" :disabled="!results.length" @click="copyAll">复制全部</BaseButton>
      <BaseButton variant="ghost" :disabled="!results.length" @click="exportJson">
        导出 JSON
      </BaseButton>
    </div>

    <!-- 数据表格 -->
    <div v-if="results.length" class="result-table">
      <div class="rt-head">
        <span class="rt-col-idx">#</span>
        <span class="rt-col-name">姓名</span>
        <span class="rt-col-gender">性别</span>
        <span class="rt-col-age">年龄</span>
        <span class="rt-col-ethnicity">民族</span>
        <span class="rt-col-birth">出生日期</span>
        <span class="rt-col-id">身份证号</span>
        <span class="rt-col-phone">手机号</span>
        <span class="rt-col-email">邮箱</span>
        <span class="rt-col-addr">住址</span>
        <span class="rt-col-act">操作</span>
      </div>
      <div v-for="(r, i) in results" :key="i" class="rt-row">
        <span class="rt-col-idx">{{ i + 1 }}</span>
        <span class="rt-col-name clickable" @click="copyText(r.name)">{{ r.name }}</span>
        <span class="rt-col-gender clickable" :class="r.gender" @click="copyText(r.gender)">
          {{ r.gender }}
        </span>
        <span class="rt-col-age clickable" @click="copyText(String(r.age))">{{ r.age }}</span>
        <span class="rt-col-ethnicity clickable" @click="copyText(r.ethnicity)">
          {{ r.ethnicity }}
        </span>
        <span class="rt-col-birth mono clickable" @click="copyText(r.birthday)">
          {{ r.birthday }}
        </span>
        <span
          class="rt-col-id mono clickable"
          @click="copyText(maskId ? r.idCardMasked : r.idCard)"
        >
          {{ maskId ? r.idCardMasked : r.idCard }}
        </span>
        <span
          class="rt-col-phone mono clickable"
          @click="copyText(maskPhoneToggle ? r.phoneMasked : r.phone)"
        >
          {{ maskPhoneToggle ? r.phoneMasked : r.phone }}
        </span>
        <span class="rt-col-email mono clickable" @click="copyText(r.email)">{{ r.email }}</span>
        <span class="rt-col-addr clickable" @click="copyText(r.address)">{{ r.address }}</span>
        <span class="rt-col-act">
          <button class="copy-btn" @click="copyRow(r)">复制</button>
          <button class="copy-btn" @click="generateForRow(i)">生成</button>
        </span>
      </div>
    </div>
    <div v-else class="empty">点击「重新生成」创建测试数据</div>

    <!-- 身份证图片生成 -->
    <div v-if="results.length" class="id-gen-section">
      <div class="section-head">
        <span class="section-label">身份证图片生成（仅供开发测试）</span>
        <div class="card-actions">
          <BaseButton v-if="cardFrontUrl" variant="ghost" @click="downloadCardImage('front')">
            下载正面
          </BaseButton>
          <BaseButton v-if="cardBackUrl" variant="ghost" @click="downloadCardImage('back')">
            下载背面
          </BaseButton>
        </div>
      </div>

      <div class="card-preview-grid">
        <div class="card-preview-item">
          <div class="cp-label">人像面（正面）</div>
          <div class="card-preview">
            <div v-if="cardLoading === 'front'" class="card-loading">生成中…</div>
            <img
              v-else-if="cardFrontUrl"
              :src="cardFrontUrl"
              class="card-img"
              alt="正面"
              @click="downloadCardImage('front')"
            />
            <div v-else class="card-placeholder">
              <IdCard :size="36" />
              <p>点击「生成」</p>
            </div>
          </div>
        </div>
        <div class="card-preview-item">
          <div class="cp-label">国徽面（背面）</div>
          <div class="card-preview">
            <div v-if="cardLoading === 'back'" class="card-loading">生成中…</div>
            <img
              v-else-if="cardBackUrl"
              :src="cardBackUrl"
              class="card-img"
              alt="背面"
              @click="downloadCardImage('back')"
            />
            <div v-else class="card-placeholder">
              <IdCard :size="36" />
              <p>点击「生成」</p>
            </div>
          </div>
        </div>
      </div>

      <details class="debug-panel">
        <summary>位置调试</summary>
        <div class="debug-grid">
          <div class="debug-col">
            <div class="debug-title">正面</div>
            <div class="debug-row">
              <span>姓名</span>
              <label>
                X
                <input v-model.number="cardLayout.front.nameX" type="range" min="0" max="1887" />
              </label>
              <label>
                Y
                <input v-model.number="cardLayout.front.nameY" type="range" min="0" max="1190" />
              </label>
            </div>
            <div class="debug-row">
              <span>性别</span>
              <label>
                X
                <input v-model.number="cardLayout.front.genderX" type="range" min="0" max="1887" />
              </label>
              <label>
                Y
                <input v-model.number="cardLayout.front.genderY" type="range" min="0" max="1190" />
              </label>
            </div>
            <div class="debug-row">
              <span>民族</span>
              <label>
                X
                <input v-model.number="cardLayout.front.nationX" type="range" min="0" max="1887" />
              </label>
              <label>
                Y
                <input v-model.number="cardLayout.front.nationY" type="range" min="0" max="1190" />
              </label>
            </div>
            <div class="debug-row">
              <span>出生年</span>
              <label>
                X
                <input
                  v-model.number="cardLayout.front.birthYearX"
                  type="range"
                  min="0"
                  max="1887"
                />
              </label>
              <label>
                Y
                <input v-model.number="cardLayout.front.birthY" type="range" min="0" max="1190" />
              </label>
            </div>
            <div class="debug-row">
              <span>出生月</span>
              <label>
                X
                <input
                  v-model.number="cardLayout.front.birthMonthX"
                  type="range"
                  min="0"
                  max="1887"
                />
              </label>
            </div>
            <div class="debug-row">
              <span>出生日</span>
              <label>
                X
                <input
                  v-model.number="cardLayout.front.birthDayX"
                  type="range"
                  min="0"
                  max="1887"
                />
              </label>
            </div>
            <div class="debug-row">
              <span>住址</span>
              <label>
                X
                <input v-model.number="cardLayout.front.addrX" type="range" min="0" max="1887" />
              </label>
              <label>
                Y
                <input v-model.number="cardLayout.front.addrY" type="range" min="0" max="1190" />
              </label>
            </div>
            <div class="debug-row">
              <span>号码</span>
              <label>
                X
                <input v-model.number="cardLayout.front.idX" type="range" min="0" max="1887" />
              </label>
              <label>
                Y
                <input v-model.number="cardLayout.front.idY" type="range" min="0" max="1190" />
              </label>
            </div>
            <div class="debug-row">
              <span>头像</span>
              <label>
                X
                <input v-model.number="cardLayout.front.avatarX" type="range" min="0" max="1887" />
              </label>
              <label>
                Y
                <input v-model.number="cardLayout.front.avatarY" type="range" min="0" max="1190" />
              </label>
            </div>
            <label class="debug-color">
              文字颜色
              <input v-model="cardLayout.front.textColor" type="color" />
            </label>
          </div>
          <div class="debug-col">
            <div class="debug-title">背面</div>
            <div class="debug-row">
              <span>签发机关</span>
              <label>
                X
                <input v-model.number="cardLayout.back.issuerX" type="range" min="0" max="1887" />
              </label>
              <label>
                Y
                <input v-model.number="cardLayout.back.issuerY" type="range" min="0" max="1188" />
              </label>
            </div>
            <div class="debug-row">
              <span>有效期限</span>
              <label>
                X
                <input v-model.number="cardLayout.back.validX" type="range" min="0" max="1887" />
              </label>
              <label>
                Y
                <input v-model.number="cardLayout.back.validY" type="range" min="0" max="1188" />
              </label>
            </div>
            <label class="debug-color">
              文字颜色
              <input v-model="cardLayout.back.textColor" type="color" />
            </label>
          </div>
        </div>
        <div class="debug-output">
          <button class="debug-print" @click="printLayout">打印坐标到控制台</button>
          <button class="debug-print" @click="copyLayout">复制坐标 JSON</button>
        </div>
      </details>
    </div>
  </ToolShell>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { Users, RefreshCw, IdCard } from '@lucide/vue';
import ToolShell from '@/components/layout/ToolShell.vue';
import BaseButton from '@/components/ui/BaseButton.vue';
import { useToast } from '@/composables/useToast';
import { copyToClipboard } from '@/composables/useClipboard';
import { useToolState } from '@/composables/useToolState';

const toast = useToast();
const count = ref(5);
const maskId = ref(false);
const maskPhoneToggle = ref(false);
const avatarImg = ref<HTMLImageElement | null>(null);
const activeRowIndex = ref(0);
const cardFrontUrl = ref('');
const cardBackUrl = ref('');
const cardLoading = ref<'' | 'front' | 'back'>('');
const cardLayout = useToolState('datagen', 'cardLayout', {
  front: {
    nameX: 330,
    nameY: 213,
    genderX: 330,
    genderY: 354,
    nationX: 737,
    nationY: 358,
    birthYearX: 330,
    birthMonthX: 650,
    birthDayX: 860,
    birthY: 495,
    addrX: 330,
    addrY: 636,
    addrLineGap: 58,
    idX: 534,
    idY: 996,
    avatarX: 1295,
    avatarY: 220,
    avatarW: 430,
    avatarH: 540,
    textColor: '#000000',
    idColor: '#151515',
  },
  back: {
    issuerX: 725,
    issuerY: 853,
    validX: 725,
    validY: 999,
    textColor: '#202020',
  },
});

interface GenResult {
  name: string;
  gender: '男' | '女';
  birthday: string;
  age: number;
  ethnicity: string;
  idCard: string;
  idCardMasked: string;
  phone: string;
  phoneMasked: string;
  email: string;
  region: string;
  address: string;
  issuer: string;
  validRange: string;
}

const SURNAMES =
  '赵钱孙李周吴郑王冯陈褚卫蒋沈韩杨朱秦尤许何吕施张孔曹严华金魏陶姜戚谢邹喻柏水窦章云苏潘葛奚范彭郎鲁韦昌马苗凤花方俞任袁柳鲍史唐费廉岑薛雷贺倪汤滕殷罗毕郝邬安常乐于时傅皮卞齐康伍余元卜顾孟平黄和穆萧尹姚邵湛汪祁毛禹狄米贝明臧计伏成戴谈宋茅庞熊纪舒屈项祝董梁杜阮蓝闵席季麻强贾路娄危江童颜郭梅盛林刁钟徐邱骆高夏蔡田樊胡凌霍虞万支柯昝管卢莫经房裘缪干解应宗丁宣贲邓郁单杭洪包诸左石崔吉钮龚程嵇邢滑裴陆荣翁荀羊於惠甄曲家封芮羿储靳汲邴糜松井段富巫乌焦巴弓牧隗山谷车侯宓蓬全郗班仰秋仲伊宫';
const NAME_CHARS_MALE =
  '伟刚勇毅俊峰强军平保东文辉力明永健世广志义兴良海山仁波宁贵福生龙元全国胜学祥才发武新利清飞彬富顺信子杰涛昌成康星光天达安岩中茂进林有坚和彪博诚先敬震振壮会思群豪心邦承乐绍功松善厚庆磊民友裕河哲江超浩亮政谦亨奇固之轮翰朗伯宏言若鸣朋斌梁栋维启克伦翔旭鹏泽晨辰士以建家致树炎德行时泰盛雄琛钧冠策腾楠榕风航弘';
const NAME_CHARS_FEMALE =
  '秀娟英华慧巧美娜静淑惠珠翠雅芝玉萍红娥玲芬芳燕彩春菊兰凤洁梅琳素云莲真环雪荣爱妹霞香月莺媛艳瑞凡佳嘉琼勤珍贞莉桂娣叶璧璐娅琦晶妍茜秋珊莎锦黛青倩婷姣婉娴瑾颖露瑶怡婵雁蓓凝晓欢霄枫芸菲寒伊亚宜可彤影诗思丽';
const REGIONS: { code: string; name: string }[] = [
  { code: '110101', name: '北京市东城区' },
  { code: '110102', name: '北京市西城区' },
  { code: '110105', name: '北京市朝阳区' },
  { code: '110108', name: '北京市海淀区' },
  { code: '310101', name: '上海市黄浦区' },
  { code: '310104', name: '上海市徐汇区' },
  { code: '310115', name: '上海市浦东新区' },
  { code: '440103', name: '广州市荔湾区' },
  { code: '440106', name: '广州市天河区' },
  { code: '440304', name: '深圳市福田区' },
  { code: '440305', name: '深圳市南山区' },
  { code: '330102', name: '杭州市上城区' },
  { code: '330106', name: '杭州市西湖区' },
  { code: '320105', name: '南京市建邺区' },
  { code: '510104', name: '成都市锦江区' },
  { code: '510107', name: '成都市武侯区' },
  { code: '420102', name: '武汉市江岸区' },
  { code: '370102', name: '济南市历下区' },
  { code: '610104', name: '西安市莲湖区' },
  { code: '500103', name: '重庆市渝中区' },
  { code: '120101', name: '天津市和平区' },
  { code: '440303', name: '深圳市罗湖区' },
  { code: '320102', name: '南京市玄武区' },
  { code: '330108', name: '杭州市滨江区' },
  { code: '210102', name: '沈阳市和平区' },
];
const PHONE_PREFIXES = [
  '130',
  '131',
  '132',
  '133',
  '134',
  '135',
  '136',
  '137',
  '138',
  '139',
  '150',
  '151',
  '152',
  '153',
  '155',
  '156',
  '157',
  '158',
  '159',
  '170',
  '176',
  '177',
  '178',
  '180',
  '181',
  '182',
  '183',
  '184',
  '185',
  '186',
  '187',
  '188',
  '189',
  '191',
  '198',
  '199',
];
const WEIGHTS = [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2];
const CHECK_CODES = ['1', '0', 'X', '9', '8', '7', '6', '5', '4', '3', '2'];

function rand(max: number) {
  return Math.floor(Math.random() * max);
}
function pick<T>(arr: T[]): T {
  return arr[rand(arr.length)]!;
}

function generateName(gender: '男' | '女'): string {
  const surname = pick([...SURNAMES]);
  const chars = gender === '男' ? NAME_CHARS_MALE : NAME_CHARS_FEMALE;
  const nameLen = rand(100) < 30 ? 1 : 2;
  let given = '';
  for (let i = 0; i < nameLen; i++) given += pick([...chars]);
  return surname + given;
}

function generateBirthday() {
  const year = 1960 + rand(45);
  const month = 1 + rand(12);
  const day = 1 + rand(28);
  const p = (n: number) => String(n).padStart(2, '0');
  return { date: new Date(year, month - 1, day), str: `${year}-${p(month)}-${p(day)}` };
}

function generateIdCard(region: string, birth: Date, gender: '男' | '女'): string {
  const p = (n: number) => String(n).padStart(2, '0');
  const birthStr = `${birth.getFullYear()}${p(birth.getMonth() + 1)}${p(birth.getDate())}`;
  let seq = rand(900) + 100;
  if (gender === '男' && seq % 2 === 0) seq++;
  if (gender === '女' && seq % 2 === 1) seq++;
  const body = region + birthStr + seq;
  let sum = 0;
  for (let i = 0; i < 17; i++) sum += parseInt(body[i]!, 10) * WEIGHTS[i]!;
  return body + CHECK_CODES[sum % 11]!;
}

function generatePhone(): string {
  let suffix = '';
  for (let i = 0; i < 8; i++) suffix += rand(10);
  return pick(PHONE_PREFIXES) + suffix;
}

const ETHNICITIES = [
  '汉',
  '壮',
  '回',
  '满',
  '维吾尔',
  '苗',
  '彝',
  '土家',
  '藏',
  '蒙古',
  '侗',
  '布依',
  '瑶',
  '白',
  '朝鲜',
];
const EMAIL_DOMAINS = [
  'qq.com',
  '163.com',
  'gmail.com',
  'outlook.com',
  '126.com',
  'sina.com',
  'foxmail.com',
];

function generateEmail(): string {
  const pinyin = Math.random().toString(36).slice(2, 8);
  const prefix = rand(3) === 0 ? pinyin : pinyin + rand(100);
  return `${prefix}@${pick(EMAIL_DOMAINS)}`;
}

function maskIdCardStr(id: string) {
  return id.length < 11 ? id : id.slice(0, 4) + '**********' + id.slice(-4);
}
function maskPhoneStr(phone: string) {
  return phone.length < 7 ? phone : phone.slice(0, 3) + '****' + phone.slice(-4);
}

function generateOne(): GenResult {
  const gender: '男' | '女' = rand(2) === 0 ? '男' : '女';
  const { date, str: birthday } = generateBirthday();
  const ri = pick(REGIONS);
  const idCard = generateIdCard(ri.code, date, gender);
  const phone = generatePhone();
  const issueYear = date.getFullYear() + 16 + rand(8);
  const name = generateName(gender);
  const birthYear = date.getFullYear();
  const age = new Date().getFullYear() - birthYear;
  return {
    name,
    gender,
    birthday,
    age,
    ethnicity: rand(10) === 0 ? pick(ETHNICITIES.slice(1)) : '汉',
    idCard,
    idCardMasked: maskIdCardStr(idCard),
    phone,
    phoneMasked: maskPhoneStr(phone),
    email: generateEmail(),
    region: ri.name,
    address: ri.name + rand(20) + 1 + '号',
    issuer: ri.name.slice(0, 2) + '市公安局' + ri.name.slice(2) + '分局',
    validRange: `${issueYear}.${String(rand(12) + 1).padStart(2, '0')}.${String(rand(28) + 1).padStart(2, '0')}-${issueYear + 20}.${String(rand(12) + 1).padStart(2, '0')}.${String(rand(28) + 1).padStart(2, '0')}`,
  };
}

const results = ref<GenResult[]>([]);

function generate() {
  results.value = Array.from({ length: count.value }, () => generateOne());
  cardFrontUrl.value = '';
  cardBackUrl.value = '';
  toast.success(`已生成 ${count.value} 条测试数据`);
}

const birthDisplay = computed(() => {
  const r = results.value[0];
  if (!r) return '';
  const [y, m, d] = r.birthday.split('-');
  return `${y}年${m}月${d}日`;
});
void birthDisplay.value;

function copyRow(r: GenResult) {
  const json = JSON.stringify(
    {
      name: r.name,
      gender: r.gender,
      age: r.age,
      ethnicity: r.ethnicity,
      birthday: r.birthday,
      idCard: maskId.value ? r.idCardMasked : r.idCard,
      phone: maskPhoneToggle.value ? r.phoneMasked : r.phone,
      email: r.email,
      region: r.region,
      address: r.address,
    },
    null,
    2,
  );
  copyToClipboard(json, '已复制 JSON');
}

function copyAll() {
  const lines = results.value.map(
    (r) =>
      `${r.name}\t${r.gender}\t${r.birthday}\t${maskId.value ? r.idCardMasked : r.idCard}\t${maskPhoneToggle.value ? r.phoneMasked : r.phone}\t${r.region}`,
  );
  copyToClipboard(lines.join('\n'), `已复制 ${lines.length} 条`);
}

function exportJson() {
  const data = results.value.map((r) => ({
    name: r.name,
    gender: r.gender,
    age: r.age,
    ethnicity: r.ethnicity,
    birthday: r.birthday,
    idCard: maskId.value ? r.idCardMasked : r.idCard,
    phone: maskPhoneToggle.value ? r.phoneMasked : r.phone,
    email: r.email,
    region: r.region,
    address: r.address,
  }));
  const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `test-data-${Date.now()}.json`;
  a.click();
  URL.revokeObjectURL(url);
  toast.success('已导出 JSON');
}

async function copyText(text: string) {
  await copyToClipboard(text, '已复制');
}

// ====== 身份证图片 Canvas 绘制 ======
async function loadAvatarByGender(gender: '男' | '女') {
  const prefix = gender === '男' ? 'male' : 'female';
  const idx = rand(5) + 1;
  return new Promise<void>((resolve) => {
    const img = new Image();
    img.onload = () => {
      avatarImg.value = img;
      resolve();
    };
    img.onerror = () => resolve();
    img.src = `/idcard/avatars/${prefix}-${idx}.png`;
  });
}

async function loadFont(family: string, url: string): Promise<FontFace> {
  const buf = await (await fetch(url)).arrayBuffer();
  const ff = new FontFace(family, buf);
  await ff.load();
  document.fonts.add(ff);
  return ff;
}

let fontsLoaded = false;
async function ensureFonts() {
  if (fontsLoaded) return;
  await Promise.all([
    loadFont('id-hei', '/idcard/hei.ttf'),
    loadFont('id-fzhei', '/idcard/fzhei.ttf'),
    loadFont('id-ocrb', '/idcard/ocrb10bt.ttf'),
  ]);
  fontsLoaded = true;
}

async function generateCardImage(side: 'front' | 'back', rowIndex?: number, silent = false) {
  const idx = rowIndex ?? activeRowIndex.value;
  const r = results.value[idx];
  if (!r) return;
  cardLoading.value = side;
  try {
    await ensureFonts();
    const tmplSrc = side === 'front' ? '/idcard/人像.png' : '/idcard/国徽.png';
    const tmpl = new Image();
    tmpl.src = tmplSrc;
    await new Promise<void>((resolve, reject) => {
      tmpl.onload = () => resolve();
      tmpl.onerror = () => reject(new Error('模板加载失败'));
    });

    const W = tmpl.naturalWidth;
    const H = tmpl.naturalHeight;
    const canvas = document.createElement('canvas');
    canvas.width = W;
    canvas.height = H;
    const ctx = canvas.getContext('2d')!;

    ctx.drawImage(tmpl, 0, 0);

    const sx = W / 1887;
    const sy = H / (side === 'front' ? 1190 : 1188);
    const X = (px: number) => px * sx;
    const Y = (py: number) => py * sy;
    const FS = (px: number) => px * sx;

    const [y, m, d] = r.birthday.split('-');
    const idDisplay = maskId.value ? r.idCardMasked : r.idCard;

    ctx.fillStyle = '#000';
    ctx.textBaseline = 'top';

    if (side === 'front') {
      ctx.fillStyle = cardLayout.value.front.textColor;

      // 姓名
      ctx.font = `${FS(58)}px id-hei`;
      ctx.fillText(r.name, X(cardLayout.value.front.nameX), Y(cardLayout.value.front.nameY));

      // 性别 / 民族
      ctx.font = `${FS(46)}px id-hei`;
      ctx.fillText(r.gender, X(cardLayout.value.front.genderX), Y(cardLayout.value.front.genderY));
      ctx.fillText('汉', X(cardLayout.value.front.nationX), Y(cardLayout.value.front.nationY));

      // 出生日期
      ctx.font = `${FS(42)}px id-fzhei`;
      ctx.fillText(y!, X(cardLayout.value.front.birthYearX), Y(cardLayout.value.front.birthY));
      ctx.fillText(m!, X(cardLayout.value.front.birthMonthX), Y(cardLayout.value.front.birthY));
      ctx.fillText(d!, X(cardLayout.value.front.birthDayX), Y(cardLayout.value.front.birthY));

      // 住址（两行）
      ctx.font = `${FS(42)}px id-hei`;
      const addr = r.address;
      let addrLoc = cardLayout.value.front.addrY;
      let addrStart = 0;
      const maxLine = 12;
      while (addrStart + maxLine < addr.length) {
        ctx.fillText(
          addr.slice(addrStart, addrStart + maxLine),
          X(cardLayout.value.front.addrX),
          Y(addrLoc),
        );
        addrStart += maxLine;
        addrLoc += cardLayout.value.front.addrLineGap;
      }
      ctx.fillText(addr.slice(addrStart), X(cardLayout.value.front.addrX), Y(addrLoc));

      // 身份证号（模板里为深色）
      ctx.fillStyle = cardLayout.value.front.idColor;
      ctx.font = `${FS(54)}px id-ocrb`;
      ctx.fillText(idDisplay, X(cardLayout.value.front.idX), Y(cardLayout.value.front.idY));

      if (avatarImg.value) {
        const avW = cardLayout.value.front.avatarW * sx;
        const avH = cardLayout.value.front.avatarH * sy;
        const avX = X(cardLayout.value.front.avatarX);
        const avY = Y(cardLayout.value.front.avatarY);
        const img = avatarImg.value;
        const ratio = Math.min(avW / img.width, avH / img.height);
        const drawW = img.width * ratio;
        const drawH = img.height * ratio;
        const offX = avX + (avW - drawW) / 2;
        const offY = avY + (avH - drawH) / 2;
        ctx.drawImage(img, offX, offY, drawW, drawH);
      }
    } else {
      ctx.fillStyle = cardLayout.value.back.textColor;
      ctx.font = `${FS(42)}px id-hei`;
      ctx.fillText(r.issuer, X(cardLayout.value.back.issuerX), Y(cardLayout.value.back.issuerY));
      ctx.fillText(r.validRange, X(cardLayout.value.back.validX), Y(cardLayout.value.back.validY));
    }

    const url = canvas.toDataURL('image/png');
    if (side === 'front') {
      cardFrontUrl.value = url;
    } else {
      cardBackUrl.value = url;
    }
    if (!silent) toast.success(`身份证${side === 'front' ? '正面' : '背面'}已生成`);
  } catch (e) {
    if (!silent) toast.error('生成失败: ' + e);
  } finally {
    cardLoading.value = '';
  }
}

function downloadCardImage(side: 'front' | 'back') {
  const url = side === 'front' ? cardFrontUrl.value : cardBackUrl.value;
  if (!url) return;
  const a = document.createElement('a');
  a.href = url;
  a.download = `idcard-${side === 'front' ? '正面' : '背面'}-${Date.now()}.png`;
  a.click();
  toast.success('已下载');
}

async function generateForRow(idx: number) {
  const r = results.value[idx];
  if (!r) return;
  activeRowIndex.value = idx;
  await loadAvatarByGender(r.gender);
  await generateCardImage('front', idx);
  await generateCardImage('back', idx);
}

generate();

function printLayout() {
  console.log('===== 身份证坐标 =====');
  const f = cardLayout.value.front;
  console.log('正面:', {
    姓名: `X:${f.nameX} Y:${f.nameY}`,
    性别: `X:${f.genderX} Y:${f.genderY}`,
    民族: `X:${f.nationX} Y:${f.nationY}`,
    出生年: `X:${f.birthYearX} Y:${f.birthY}`,
    出生月: `X:${f.birthMonthX}`,
    出生日: `X:${f.birthDayX}`,
    住址: `X:${f.addrX} Y:${f.addrY}`,
    号码: `X:${f.idX} Y:${f.idY}`,
    头像: `X:${f.avatarX} Y:${f.avatarY} W:${f.avatarW} H:${f.avatarH}`,
    textColor: f.textColor,
    idColor: f.idColor,
  });
  const b = cardLayout.value.back;
  console.log('背面:', {
    签发机关: `X:${b.issuerX} Y:${b.issuerY}`,
    有效期限: `X:${b.validX} Y:${b.validY}`,
    textColor: b.textColor,
  });
  console.log('完整 JSON:', JSON.stringify(cardLayout.value, null, 2));
  toast.success('坐标已打印到浏览器控制台 (F12)');
}

async function copyLayout() {
  const text = JSON.stringify(cardLayout.value, null, 2);
  await copyToClipboard(text, '坐标 JSON 已复制');
}

watch(
  () => JSON.stringify(cardLayout.value),
  () => {
    if (cardFrontUrl.value) generateCardImage('front', undefined, true);
    if (cardBackUrl.value) generateCardImage('back', undefined, true);
  },
);
</script>

<style scoped>
.gen-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  margin-bottom: 16px;
}
.opt-group {
  display: flex;
  align-items: center;
  gap: 6px;
}
.opt-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--xuya-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.4px;
}
.opt-select {
  padding: 5px 10px;
  font-size: 12px;
  color: var(--xuya-text);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  outline: none;
}
.sep {
  width: 1px;
  height: 20px;
  background: var(--xuya-border);
  margin: 0 2px;
}
.toggle {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  color: var(--xuya-text-secondary);
  cursor: pointer;
}
.toggle input {
  accent-color: var(--xuya-accent);
  width: 14px;
  height: 14px;
  cursor: pointer;
}

.result-table {
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  overflow-x: auto;
  overflow-y: auto;
  max-height: 400px;
  margin-bottom: 20px;
  flex-shrink: 0;
}
.rt-head {
  display: grid;
  grid-template-columns: 52px 76px 56px 56px 56px 108px 170px 124px 160px 200px 110px;
  gap: 6px;
  padding: 8px 12px;
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.3px;
  color: var(--xuya-text-tertiary);
  background: var(--xuya-input-bg);
  border-bottom: 1px solid var(--xuya-border);
  min-width: 1170px;
}
.rt-row {
  display: grid;
  grid-template-columns: 52px 76px 56px 56px 56px 108px 170px 124px 160px 200px 110px;
  gap: 6px;
  padding: 8px 12px;
  font-size: 12px;
  align-items: center;
  border-bottom: 1px solid var(--xuya-border-light);
  transition: background var(--xuya-duration-fast);
  min-width: 1170px;
}
.rt-row:last-child {
  border-bottom: none;
}
.rt-row:hover {
  background: var(--xuya-input-bg);
}
.rt-col-idx {
  color: var(--xuya-text-tertiary);
  font-family: var(--xuya-font-mono);
}
.rt-col-name {
  font-weight: 600;
  color: var(--xuya-text);
}
.rt-col-gender {
  font-weight: 600;
}
.rt-col-gender.男 {
  color: var(--xuya-info);
}
.rt-col-gender.女 {
  color: var(--xuya-danger);
}
.rt-col-age {
  color: var(--xuya-text-secondary);
  text-align: center;
}
.rt-col-ethnicity {
  color: var(--xuya-text);
  text-align: center;
}
.rt-col-birth {
  color: var(--xuya-text-secondary);
  font-size: 11px;
}
.rt-col-id {
  color: var(--xuya-accent);
  font-weight: 600;
}
.rt-col-phone {
  color: var(--xuya-text);
}
.rt-col-email {
  color: var(--xuya-text);
  font-size: 11px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.rt-col-addr {
  color: var(--xuya-text-tertiary);
  font-size: 11px;
}
.clickable {
  cursor: pointer;
}
.clickable:hover {
  text-decoration: underline;
}
.rt-col-act {
  display: flex;
  align-items: center;
  gap: 4px;
}
.copy-btn {
  padding: 2px 8px;
  font-size: 11px;
  color: var(--xuya-text-secondary);
  background: transparent;
  border: 1px solid var(--xuya-border);
  border-radius: 4px;
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
}
.copy-btn:hover {
  color: var(--xuya-accent);
  border-color: var(--xuya-accent);
}
.mono {
  font-family: var(--xuya-font-mono);
}
.empty {
  padding: 40px;
  text-align: center;
  color: var(--xuya-text-tertiary);
  font-size: 14px;
}

.id-gen-section {
  margin-top: 8px;
}
.section-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
  gap: 10px;
  flex-wrap: wrap;
}
.section-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--xuya-text-tertiary);
}
.card-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.card-preview-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}
.card-preview-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.cp-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
}
.card-preview {
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-lg);
  background: var(--xuya-input-bg);
  min-height: 200px;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  padding: 12px;
}
.card-loading {
  font-size: 14px;
  color: var(--xuya-text-tertiary);
}
.card-img {
  max-width: 100%;
  max-height: 380px;
  cursor: pointer;
  transition: transform var(--xuya-duration-fast);
}
.card-img:hover {
  transform: scale(1.02);
}
.card-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  color: var(--xuya-text-tertiary);
}
.card-placeholder p {
  font-size: 13px;
}
.card-placeholder .sub {
  font-size: 11px;
  opacity: 0.7;
}

.debug-panel {
  margin-top: 14px;
  background: var(--xuya-card-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius);
  padding: 12px 14px;
}
.debug-panel summary {
  cursor: pointer;
  font-size: 13px;
  font-weight: 600;
  color: var(--xuya-text-secondary);
}
.debug-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
  margin-top: 12px;
}
.debug-col {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.debug-title {
  font-size: 11px;
  font-weight: 700;
  color: var(--xuya-text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.4px;
}
.debug-row {
  display: grid;
  grid-template-columns: 48px 1fr 1fr 1fr;
  gap: 8px;
  align-items: center;
  font-size: 11px;
  color: var(--xuya-text-secondary);
}
.debug-row input[type='range'] {
  width: 100%;
  accent-color: var(--xuya-accent);
}
.debug-color {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 11px;
  color: var(--xuya-text-secondary);
}
.debug-color input[type='color'] {
  width: 32px;
  height: 24px;
  border: 1px solid var(--xuya-border);
  border-radius: 4px;
  background: none;
  cursor: pointer;
}
.debug-output {
  display: flex;
  gap: 8px;
  margin-top: 10px;
}
.debug-print {
  padding: 5px 14px;
  font-size: 12px;
  font-weight: 500;
  color: var(--xuya-text);
  background: var(--xuya-input-bg);
  border: 1px solid var(--xuya-border);
  border-radius: var(--xuya-radius-sm);
  cursor: pointer;
  transition: all var(--xuya-duration-fast);
}
.debug-print:hover {
  color: var(--xuya-accent);
  border-color: var(--xuya-accent);
}

@media (max-width: 640px) {
  .result-table {
    overflow-x: auto;
  }
  .rt-head,
  .rt-row {
    min-width: 960px;
  }
  .debug-grid {
    grid-template-columns: 1fr;
  }
  .debug-row {
    grid-template-columns: 48px 1fr;
  }
}
</style>
