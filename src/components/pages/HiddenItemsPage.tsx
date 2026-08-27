import { useState, useEffect } from "react";
import { EyeOff, Folder, FileLock2, RefreshCw, Loader2, Search } from "lucide-react";
import { listHiddenBlobs } from "../../lib/tauri-bridge";
import { SectionHeader } from "../shared/SectionHeader";

export function HiddenItemsPage() {
  const [blobs, setBlobs] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [search, setSearch] = useState("");

  const refresh = async () => {
    setLoading(true);
    setError("");
    try {
      const items = await listHiddenBlobs();
      setBlobs(items);
    } catch (e: any) {
      setError("Failed to scan hidden items: " + e);
    }
    setLoading(false);
  };

  useEffect(() => {
    refresh();
  }, []);

  const filtered = blobs.filter(b => b.toLowerCase().includes(search.toLowerCase()));

  const formatPath = (p: string) => {
    const parts = p.replace(/\\/g, "/").split("/");
    return parts[parts.length - 1] || parts[parts.length - 2] || p;
  };

  const formatDir = (p: string) => {
    const parts = p.replace(/\\/g, "/").split("/");
    return parts.slice(0, -1).join("\\") || "\\";
  };

  return (
    <div className="max-w-6xl mx-auto space-y-6">
      <SectionHeader
        eyebrow="v0.0.39"
        title="Hidden Items"
        subtitle="Encrypted .InnologyLock blobs invisible in Explorer. Only visible here and when 'Show hidden files' is enabled in Windows."
      />

      {error && (
        <div className="p-3 rounded-lg bg-[color:var(--destructive)]/15 border border-[color:var(--destructive)]/30 text-sm text-[color:var(--destructive)]">
          {error}
        </div>
      )}

      <div className="glass rounded-2xl overflow-hidden">
        <div className="p-5 border-b border-[color:var(--border)] flex items-center gap-3">
          <EyeOff className="w-5 h-5 text-[color:var(--primary)]" />
          <h3 className="font-semibold">Encrypted Blobs</h3>
          <span className="text-xs text-[color:var(--muted-foreground)]">{blobs.length} items</span>
          <div className="ml-auto flex items-center gap-2">
            {blobs.length > 0 && (
              <div className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-surface border border-surface-border">
                <Search className="w-3.5 h-3.5 text-[color:var(--muted-foreground)]" />
                <input
                  placeholder="Search..."
                  value={search}
                  onChange={e => setSearch(e.target.value)}
                  className="bg-transparent outline-none text-xs w-32 placeholder:text-[color:var(--muted-foreground)]"
                />
              </div>
            )}
            <button
              onClick={refresh}
              disabled={loading}
              className="px-3 py-2 rounded-lg text-sm bg-surface border border-surface-border flex items-center gap-2 hover:bg-surface-active disabled:opacity-40"
            >
              {loading ? <Loader2 className="w-4 h-4 animate-spin" /> : <RefreshCw className="w-4 h-4" />}
              {loading ? "Scanning..." : "Scan"}
            </button>
          </div>
        </div>
        <div className="divide-y divide-surface-border">
          {filtered.map(blob => {
            const isVaultFile = blob.endsWith(".vaultfile");
            const isInnologyLock = blob.endsWith(".InnologyLock");
            return (
              <div key={blob} className="px-5 py-4 flex items-center gap-4">
                <div className="w-10 h-10 rounded-lg grid place-items-center bg-surface border border-surface-border">
                  {isVaultFile ? (
                    <FileLock2 className="w-4 h-4 text-[color:var(--violet)]" />
                  ) : (
                    <Folder className="w-4 h-4 text-[color:var(--primary)]" />
                  )}
                </div>
                <div className="flex-1 min-w-0">
                  <code className="text-sm">{formatPath(blob)}</code>
                  <div className="text-xs text-[color:var(--muted-foreground)] mt-0.5">
                    {isVaultFile ? "Vault Storage" : "Locked File"} in {formatDir(blob)}
                  </div>
                </div>
                <span className="text-[9px] uppercase px-2 py-1 rounded bg-surface-active text-[color:var(--muted-foreground)]">
                  {isVaultFile ? "OVLF" : "OLCK"}
                </span>
              </div>
            );
          })}
          {blobs.length === 0 && !loading && (
            <div className="p-8 text-center text-[color:var(--muted-foreground)] text-sm">
              No hidden items found. Lock files or folders from the Vault page to see them here.
            </div>
          )}
          {search && filtered.length === 0 && blobs.length > 0 && (
            <div className="p-8 text-center text-[color:var(--muted-foreground)] text-sm">
              No items match "{search}".
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
