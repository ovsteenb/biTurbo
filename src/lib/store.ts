import { create } from "zustand";
import type {
  Memory,
  Project,
  Stats,
  AgentEntry,
  ActivityEntry,
  GraphData,
} from "./types";
import { api } from "./api";

export type View = "overview" | "memories" | "projects" | "graph" | "agents" | "settings";

interface AppStore {
  view: View;
  setView: (v: View) => void;

  projects: Project[];
  currentProjectId: string;
  setCurrentProjectId: (id: string) => void;
  refreshProjects: () => Promise<void>;

  agents: AgentEntry[];
  refreshAgents: () => Promise<void>;

  stats: Stats | null;
  refreshStats: () => Promise<void>;

  activity: ActivityEntry[];
  refreshActivity: () => Promise<void>;

  searchQuery: string;
  setSearchQuery: (q: string) => void;

  tags: [string, number][];
  refreshTags: () => Promise<void>;

  memories: Memory[];
  selectedMemoryUid: string | null;
  setSelectedMemoryUid: (uid: string | null) => void;
  memoryOffset: number;
  hasMoreMemories: boolean;
  loadMoreMemories: () => Promise<void>;
  refreshMemories: () => Promise<void>;

  quickAddOpen: boolean;
  setQuickAddOpen: (open: boolean) => void;

  toast: { kind: "ok" | "err" | "info"; text: string } | null;
  showToast: (t: { kind: "ok" | "err" | "info"; text: string }) => void;
  clearToast: () => void;

  graph: GraphData | null;
  refreshGraph: () => Promise<void>;
}

export const useApp = create<AppStore>((set, get) => ({
  view: "overview",
  setView: (v) => set({ view: v }),

  projects: [],
  currentProjectId: "default",
  setCurrentProjectId: (id) => set({ currentProjectId: id, selectedMemoryUid: null }),
  refreshProjects: async () => {
    const projects = await api.listProjects();
    set({ projects });
    if (!get().currentProjectId && projects.length) {
      set({ currentProjectId: projects[0].id });
    }
  },

  agents: [],
  refreshAgents: async () => set({ agents: await api.listAgents() }),

  stats: null,
  refreshStats: async () => set({ stats: await api.stats() }),

  activity: [],
  refreshActivity: async () => set({ activity: await api.recentActivity(60) }),

  searchQuery: "",
  setSearchQuery: (q) => set({ searchQuery: q }),

  tags: [],
  refreshTags: async () => {
    const tags = await api.listTags(get().currentProjectId);
    set({ tags });
  },

  memories: [],
  selectedMemoryUid: null,
  setSelectedMemoryUid: (uid) => set({ selectedMemoryUid: uid }),
  memoryOffset: 0,
  hasMoreMemories: false,
  loadMoreMemories: async () => {
    const offset = get().memoryOffset;
    const batch = await api.listMemories({
      project_id: get().currentProjectId,
      limit: 50,
      offset,
    });
    set((s) => ({
      memories: [...s.memories, ...batch],
      memoryOffset: offset + batch.length,
      hasMoreMemories: batch.length === 50,
    }));
  },
  refreshMemories: async () => {
    const mems = await api.listMemories({
      project_id: get().currentProjectId,
      limit: 50,
      offset: 0,
    });
    set({
      memories: mems,
      memoryOffset: mems.length,
      hasMoreMemories: mems.length === 50,
    });
  },

  quickAddOpen: false,
  setQuickAddOpen: (open) => set({ quickAddOpen: open }),

  toast: null,
  showToast: (t) => {
    set({ toast: t });
    setTimeout(() => {
      if (get().toast === t) set({ toast: null });
    }, 3500);
  },
  clearToast: () => set({ toast: null }),

  graph: null,
  refreshGraph: async () => {
    const g = await api.getProjectGraph(get().currentProjectId);
    set({ graph: g });
  },
}));
