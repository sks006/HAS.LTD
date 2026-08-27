import { create } from 'zustand';

interface GlobalState {
  theme: 'light' | 'dark';
  toggleTheme: () => void;
}

export const useGlobalStore = create<GlobalState>((set) => ({
  theme: 'dark',
  toggleTheme: () => set((state) => ({ theme: state.theme === 'light' ? 'dark' : 'light' })),
}));
