import { ref, watch } from 'vue';

export type LayoutMode = 'sidebar' | 'tabbar' | 'grid' | 'dual';

export interface AccentTheme {
  id: string;
  label: string;
  light: string;
  dark: string;
  gradient: string;
}

const ACCENTS: AccentTheme[] = [
  {
    id: 'teal',
    label: 'Teal',
    light: '#0d9488',
    dark: '#2dd4bf',
    gradient: 'linear-gradient(135deg, #2dd4bf 0%, #14b8a6 100%)',
  },
  {
    id: 'blue',
    label: 'Blue',
    light: '#2563eb',
    dark: '#60a5fa',
    gradient: 'linear-gradient(135deg, #60a5fa 0%, #3b82f6 100%)',
  },
  {
    id: 'purple',
    label: 'Purple',
    light: '#7c3aed',
    dark: '#a78bfa',
    gradient: 'linear-gradient(135deg, #a78bfa 0%, #8b5cf6 100%)',
  },
  {
    id: 'green',
    label: 'Green',
    light: '#059669',
    dark: '#34d399',
    gradient: 'linear-gradient(135deg, #34d399 0%, #10b981 100%)',
  },
  {
    id: 'orange',
    label: 'Orange',
    light: '#ea580c',
    dark: '#fb923c',
    gradient: 'linear-gradient(135deg, #fb923c 0%, #f97316 100%)',
  },
  {
    id: 'pink',
    label: 'Pink',
    light: '#db2777',
    dark: '#f472b6',
    gradient: 'linear-gradient(135deg, #f472b6 0%, #ec4899 100%)',
  },
  {
    id: 'red',
    label: 'Red',
    light: '#dc2626',
    dark: '#f87171',
    gradient: 'linear-gradient(135deg, #f87171 0%, #ef4444 100%)',
  },
  {
    id: 'cyan',
    label: 'Cyan',
    light: '#0891b2',
    dark: '#22d3ee',
    gradient: 'linear-gradient(135deg, #22d3ee 0%, #06b6d4 100%)',
  },
  {
    id: 'indigo',
    label: 'Indigo',
    light: '#4f46e5',
    dark: '#818cf8',
    gradient: 'linear-gradient(135deg, #818cf8 0%, #6366f1 100%)',
  },
  {
    id: 'amber',
    label: 'Amber',
    light: '#d97706',
    dark: '#fbbf24',
    gradient: 'linear-gradient(135deg, #fbbf24 0%, #f59e0b 100%)',
  },
  {
    id: 'lime',
    label: 'Lime',
    light: '#65a30d',
    dark: '#a3e635',
    gradient: 'linear-gradient(135deg, #a3e635 0%, #84cc16 100%)',
  },
  {
    id: 'emerald',
    label: 'Emerald',
    light: '#059669',
    dark: '#10b981',
    gradient: 'linear-gradient(135deg, #6ee7b7 0%, #10b981 100%)',
  },
  {
    id: 'rose',
    label: 'Rose',
    light: '#e11d48',
    dark: '#fb7185',
    gradient: 'linear-gradient(135deg, #fb7185 0%, #f43f5e 100%)',
  },
  {
    id: 'slate',
    label: 'Slate',
    light: '#475569',
    dark: '#94a3b8',
    gradient: 'linear-gradient(135deg, #94a3b8 0%, #64748b 100%)',
  },
];

const LAYOUT_KEY = 'xuya_layout_mode';
const ACCENT_KEY = 'xuya_accent_id';

const layoutMode = ref<LayoutMode>((localStorage.getItem(LAYOUT_KEY) as LayoutMode) || 'sidebar');
const accentId = ref<string>(localStorage.getItem(ACCENT_KEY) || 'teal');

function hexToRgb(hex: string): { r: number; g: number; b: number } {
  const m = hex.match(/^#?([\da-f]{2})([\da-f]{2})([\da-f]{2})$/i);
  if (!m) return { r: 13, g: 148, b: 136 };
  return { r: parseInt(m[1]!, 16), g: parseInt(m[2]!, 16), b: parseInt(m[3]!, 16) };
}

function applyAccent(id: string) {
  const theme = ACCENTS.find((a) => a.id === id) ?? ACCENTS[0]!;
  const root = document.documentElement;
  const light = hexToRgb(theme.light);
  const dark = hexToRgb(theme.dark);

  root.style.setProperty('--xuya-accent', theme.light);
  root.style.setProperty('--xuya-accent-hover', theme.dark);
  root.style.setProperty('--xuya-accent-2', theme.dark);
  root.style.setProperty('--xuya-accent-gradient', theme.gradient);
  root.style.setProperty('--xuya-accent-gradient-hover', theme.gradient);
  root.style.setProperty('--xuya-accent-soft', `rgba(${light.r}, ${light.g}, ${light.b}, 0.08)`);
  root.style.setProperty(
    '--xuya-accent-soft-strong',
    `rgba(${light.r}, ${light.g}, ${light.b}, 0.14)`,
  );
  root.style.setProperty('--xuya-accent-ring', `rgba(${light.r}, ${light.g}, ${light.b}, 0.15)`);
  root.style.setProperty('--xuya-accent-glow', `rgba(${light.r}, ${light.g}, ${light.b}, 0.25)`);

  const darkStyle = root.querySelector('#dynamic-dark-accent');
  if (darkStyle) darkStyle.remove();
  const style = document.createElement('style');
  style.id = 'dynamic-dark-accent';
  style.textContent = `
    .dark-theme {
      --xuya-accent: ${theme.dark} !important;
      --xuya-accent-hover: ${theme.dark} !important;
      --xuya-accent-2: ${theme.dark} !important;
      --xuya-accent-soft: rgba(${dark.r}, ${dark.g}, ${dark.b}, 0.1) !important;
      --xuya-accent-soft-strong: rgba(${dark.r}, ${dark.g}, ${dark.b}, 0.16) !important;
      --xuya-accent-ring: rgba(${dark.r}, ${dark.g}, ${dark.b}, 0.18) !important;
      --xuya-accent-glow: rgba(${dark.r}, ${dark.g}, ${dark.b}, 0.3) !important;
    }
  `;
  root.appendChild(style);
}

export function initLayout() {
  applyAccent(accentId.value);
}

watch(layoutMode, (m) => localStorage.setItem(LAYOUT_KEY, m));
watch(accentId, (id) => {
  localStorage.setItem(ACCENT_KEY, id);
  applyAccent(id);
});

export function useLayout() {
  function setLayout(m: LayoutMode) {
    layoutMode.value = m;
  }

  function setAccent(id: string) {
    accentId.value = id;
  }

  return {
    layoutMode,
    accentId,
    accents: ACCENTS,
    setLayout,
    setAccent,
  };
}
