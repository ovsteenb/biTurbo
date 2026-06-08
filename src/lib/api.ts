import { invoke } from "@tauri-apps/api/core";
import type {
  Memory,
  MemoryWithScore,
  Project,
  AgentEntry,
  ActivityEntry,
  Stats,
  IngestJob,
  ConsolidateReport,
  ConsolidateStatus,
  GraphData,
  BootstrapPayload,
} from "./types";

export interface RememberInput {
  content: string;
  mem_type?: string | null;
  project_id?: string | null;
  tags?: string[] | null;
  importance?: number | null;
  source_agent?: string | null;
  supersedes?: string | null;
  file_path?: string | null;
  start_line?: number | null;
  end_line?: number | null;
  language?: string | null;
}

export interface UpdateInput {
  content?: string | null;
  mem_type?: string | null;
  tags?: string[] | null;
  importance?: number | null;
}

export const api = {
  ping: () => invoke<string>("ping"),

  listMemories: (params: {
    project_id?: string | null;
    mem_type?: string | null;
    limit?: number;
    offset?: number;
  }) =>
    invoke<Memory[]>("list_memories", {
      projectId: params.project_id ?? null,
      memType: params.mem_type ?? null,
      limit: params.limit ?? 50,
      offset: params.offset ?? 0,
    }),

  getMemory: (uid: string) =>
    invoke<Memory | null>("get_memory", { uid }),

  remember: (input: RememberInput) =>
    invoke<Memory>("remember", { input }),

  forget: (uid: string) =>
    invoke<boolean>("forget_memory", { uid }),

  update: (uid: string, input: UpdateInput) =>
    invoke<Memory>("update_memory", { uid, input }),

  search: (params: {
    project_id?: string | null;
    query: string;
    k?: number;
    mem_type?: string | null;
  }) =>
    invoke<MemoryWithScore[]>("search_memories", {
      args: {
        project_id: params.project_id ?? null,
        query: params.query,
        k: params.k ?? 10,
        mem_type: params.mem_type ?? null,
      },
    }),

  listProjects: () => invoke<Project[]>("list_projects"),
  getProject: (id: string) => invoke<Project>("get_project", { id }),
  listTags: (project_id?: string | null) =>
    invoke<[string, number][]>("list_tags", { projectId: project_id ?? null }),

  createProject: (input: {
    name: string;
    id?: string | null;
    description?: string | null;
    root_path?: string | null;
    bit_width?: number | null;
  }) =>
    invoke<Project>("create_project", {
      input: {
        name: input.name,
        id: input.id ?? null,
        description: input.description ?? null,
        root_path: input.root_path ?? null,
        bit_width: input.bit_width ?? null,
      },
    }),

  deleteProject: (id: string) => invoke<void>("delete_project", { id }),

  ingestProject: (project_id: string, root_path: string) =>
    invoke<IngestJob>("ingest_project", {
      args: { project_id, root_path },
    }),

  getProjectGraph: (project_id: string) =>
    invoke<GraphData>("get_project_graph", {
      args: { project_id },
    }),

  consolidate: (project_id?: string | null) =>
    invoke<ConsolidateReport>("consolidate_now", {
      projectId: project_id ?? null,
    }),

  consolidateStatus: () =>
    invoke<ConsolidateStatus>("consolidate_status"),

  importFolder: (project_id: string, root_path: string) =>
    invoke<{
      files_imported: number;
      memories_created: number;
      errors: string[];
    }>("import_folder", {
      args: { project_id, root_path },
    }),

  exportMemories: (project_id: string | null, output_path: string) =>
    invoke<{ memories_written: number; output_path: string }>("export_memories", {
      args: { project_id, output_path },
    }),

  setWatch: (project_id: string, root_path: string | null, enabled: boolean) =>
    invoke<{ enabled_projects: number; watching: string[] }>("set_watch", {
      args: { project_id, root_path, enabled },
    }),

  watchStatus: () =>
    invoke<{ enabled_projects: number; watching: string[] }>("watch_status"),

  setProjectEmbedModel: (project_id: string, model: string | null) =>
    invoke<void>("set_project_embed_model", {
      args: { project_id, model },
    }),

  stats: () => invoke<Stats>("stats"),
  listAgents: () => invoke<AgentEntry[]>("list_agents"),
  registerAgent: (name: string, kind: string, meta?: object) =>
    invoke<AgentEntry>("register_agent", {
      args: { name, kind, meta: meta ?? null },
    }),
  recentActivity: (limit = 100) =>
    invoke<ActivityEntry[]>("recent_activity", { limit }),

  bootstrap: () => invoke<BootstrapPayload>("bootstrap"),
};
