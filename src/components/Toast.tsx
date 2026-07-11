import { memo } from "react";
import { useApp } from "../lib/store";
import clsx from "clsx";

export const Toast = memo(function Toast() {
  const toast = useApp((s) => s.toast);
  const clear = useApp((s) => s.clearToast);
  if (!toast) return null;
  return (
    <div
      onClick={clear}
      role="status"
      aria-live="polite"
      aria-atomic="true"
      tabIndex={0}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          clear();
        }
      }}
      className={clsx(
        "fixed bottom-4 left-1/2 z-50 -translate-x-1/2 cursor-pointer rounded-md border px-3 py-2 text-sm shadow-lg animate-fade_in",
        toast.kind === "ok" && "border-success/30 bg-success/10 text-success",
        toast.kind === "err" && "border-danger/30 bg-danger/10 text-danger",
        toast.kind === "info" && "border-border bg-surface text-text"
      )}
    >
      {toast.text}
    </div>
  );
});
