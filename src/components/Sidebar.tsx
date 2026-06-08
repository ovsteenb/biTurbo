import {
  LayoutGrid,
  Brain,
  FolderGit2,
  Share2,
  Bot,
  Settings as SettingsIcon,
} from "lucide-react";
import { useApp } from "../lib/store";
import clsx from "clsx";

const nav = [
  { id: "overview", label: "Overview", icon: LayoutGrid },
  { id: "memories", label: "Memories", icon: Brain },
  { id: "projects", label: "Projects", icon: FolderGit2 },
  { id: "graph", label: "Graph", icon: Share2 },
  { id: "agents", label: "Agents", icon: Bot },
  { id: "settings", label: "Settings", icon: SettingsIcon },
] as const;

export function Sidebar() {
  const view = useApp((s) => s.view);
  const setView = useApp((s) => s.setView);
  const agents = useApp((s) => s.agents);
  const stats = useApp((s) => s.stats);

  const connectedAgents = agents.length;
  const totalMem = stats?.total_memories ?? 0;

  return (
    <aside className="flex w-60 shrink-0 flex-col border-r border-border-subtle bg-surface/40">
      {/* Brand */}
      <div className="flex h-14 items-center gap-2 border-b border-border-subtle px-4">
        <Logo />
        <div className="leading-tight">
          <div className="font-serif text-lg font-medium text-text">biTurbo</div>
          <div className="text-[10px] uppercase tracking-widest text-text-dim">
            memory layer
          </div>
        </div>
      </div>

      {/* Nav */}
      <nav className="flex-1 overflow-y-auto p-2">
        {nav.map((item) => {
          const Icon = item.icon;
          const active = view === item.id;
          return (
            <button
              key={item.id}
              onClick={() => setView(item.id as never)}
              className={clsx(
                "flex w-full items-center gap-2.5 rounded-md px-3 py-2 text-sm transition",
                active
                  ? "bg-accent-soft text-text"
                  : "text-text-muted hover:bg-surface-2 hover:text-text"
              )}
            >
              <Icon
                size={15}
                className={active ? "text-accent" : "text-text-dim"}
              />
              <span className="flex-1 text-left">{item.label}</span>
            </button>
          );
        })}

        {/* Projects sub-list */}
        <div className="mt-6 px-3">
          <div className="mb-2 flex items-center justify-between text-[10px] uppercase tracking-widest text-text-dim">
            <span>Projects</span>
            <span className="font-mono text-text-dim">
              {stats?.total_projects ?? 0}
            </span>
          </div>
          <ProjectList />
        </div>
      </nav>

      {/* Footer */}
      <div className="border-t border-border-subtle p-3">
        <div className="flex items-center gap-2 rounded-md bg-surface-2 px-3 py-2">
          <span className="relative flex h-2 w-2">
            <span className="absolute inline-flex h-full w-full animate-pulse_dot rounded-full bg-success opacity-75" />
            <span className="relative inline-flex h-2 w-2 rounded-full bg-success" />
          </span>
          <div className="flex-1 text-xs">
            <div className="font-medium text-text">
              {connectedAgents} agent{connectedAgents === 1 ? "" : "s"}
            </div>
            <div className="text-text-dim">{totalMem.toLocaleString()} memories</div>
          </div>
        </div>
      </div>
    </aside>
  );
}

function Logo() {
  return (
    <div className="relative h-7 w-7 shrink-0">
      <div className="absolute inset-0 rounded-md bg-gradient-to-br from-accent to-amber-700" />
      <div className="absolute inset-[3px] rounded-[5px] bg-bg" />
      <div className="absolute inset-[3px] flex items-center justify-center">
        <div className="h-2 w-2 rounded-full bg-accent" />
      </div>
    </div>
  );
}

function ProjectList() {
  const projects = useApp((s) => s.projects);
  const currentProjectId = useApp((s) => s.currentProjectId);
  const setCurrentProjectId = useApp((s) => s.setCurrentProjectId);
  const setView = useApp((s) => s.setView);

  return (
    <div className="space-y-0.5">
      {projects.map((p) => {
        const active = p.id === currentProjectId;
        return (
          <button
            key={p.id}
            onClick={() => {
              setCurrentProjectId(p.id);
              setView("memories");
            }}
            className={clsx(
              "group flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-xs transition",
              active
                ? "bg-surface-2 text-text"
                : "text-text-muted hover:bg-surface-2/60 hover:text-text"
            )}
          >
            <span
              className={clsx(
                "h-1.5 w-1.5 rounded-full",
                active ? "bg-accent" : "bg-text-dim group-hover:bg-text-muted"
              )}
            />
            <span className="flex-1 truncate">{p.name}</span>
            <span className="font-mono text-[10px] text-text-dim">
              {p.memory_count}
            </span>
          </button>
        );
      })}
    </div>
  );
}
