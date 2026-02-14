// Layout store for split pane management

import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { MosaicNode } from 'react-mosaic-component';

export type LayoutNode = MosaicNode<string>;

interface LayoutState {
  // Current mosaic layout
  layout: LayoutNode | null;

  // Layout presets
  presets: Record<string, LayoutNode>;

  // Actions
  setLayout: (layout: LayoutNode | null) => void;
  savePreset: (name: string, layout: LayoutNode) => void;
  loadPreset: (name: string) => void;
  deletePreset: (name: string) => void;
  resetLayout: () => void;
}

export const useLayoutStore = create<LayoutState>()(
  persist(
    (set, get) => ({
      layout: null,
      presets: {},

      setLayout: (layout) => set({ layout }),

      savePreset: (name, layout) => set((state) => ({
        presets: { ...state.presets, [name]: layout }
      })),

      loadPreset: (name) => {
        const preset = get().presets[name];
        if (preset) {
          set({ layout: preset });
        }
      },

      deletePreset: (name) => set((state) => {
        const { [name]: _, ...rest } = state.presets;
        return { presets: rest };
      }),

      resetLayout: () => set({ layout: null }),
    }),
    {
      name: 'tacoshell-layout',
    }
  )
);

