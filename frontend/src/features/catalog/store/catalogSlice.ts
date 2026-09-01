import { useStore } from '@/shared/store/store';

export const catalogSlice = {
  get activeFilter(): string { return useStore.getState().activeFilter; },
  get searchQuery(): string { return useStore.getState().searchQuery; },
  setActiveFilter: (filter: string) => useStore.getState().setActiveFilter(filter),
  setSearchQuery: (query: string) => useStore.getState().setSearchQuery(query),
};
