import { defineComponent, h, markRaw } from 'vue';
import BrandIcon from '@/components/ui/BrandIcon.vue';
import doubaoColorIconRaw from '@lobehub/icons-static-svg/icons/doubao-color.svg?raw';

const LOBE_COLOR_ICONS: Record<string, string> = {
  doubao: doubaoColorIconRaw,
};

export function createBrandIcon(iconKey: string) {
  return markRaw(
    defineComponent({
      props: {
        size: { type: Number, default: 16 },
      },
      setup(props) {
        return () =>
          h(BrandIcon, {
            iconKey,
            size: props.size,
            colorSvg: LOBE_COLOR_ICONS[iconKey],
          });
      },
    }),
  );
}
