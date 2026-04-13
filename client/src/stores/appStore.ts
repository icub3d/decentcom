import { create } from "zustand";

export interface ServerConnection {
  id: string;
  address: string;
}

interface AppStore {
  currentServerId: string | null;
  servers: Record<string, ServerConnection>;
  theme: "mocha";
  addServer: (address: string) => string;
  setCurrentServer: (id: string) => void;
}

function normalizeAddress(address: string): string {
  const trimmed = address.trim().replace(/\/$/, "");
  if (trimmed.startsWith("http://") || trimmed.startsWith("https://")) {
    return trimmed;
  }
  return `http://${trimmed}`;
}

export const useAppStore = create<AppStore>((set, get) => ({
  currentServerId: null,
  servers: {},
  theme: "mocha",
  addServer: (address) => {
    const normalized = normalizeAddress(address);
    const id = normalized;
    const existing = get().servers[id];
    if (existing) {
      set({ currentServerId: id });
      return id;
    }

    set((state) => ({
      servers: {
        ...state.servers,
        [id]: { id, address: normalized },
      },
      currentServerId: id,
    }));

    return id;
  },
  setCurrentServer: (id) => set({ currentServerId: id }),
}));
