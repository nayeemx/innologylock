import { useState, useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { Lock, Shield, X, Fingerprint, AlertTriangle } from "lucide-react";
import { widgetUnlock, widgetUnlockBiometric, hideWidget, checkBiometric, filterUnlock, filterDeny } from "../../lib/tauri-bridge";

interface UnlockTarget {
  target_type: string;
  target_id: string;
  display_name: string;
}

interface FilterAccessRequest {
  request_id: number;
  message_id: number;
  file_path: string;
  display_name: string;
}

const typeLabels: Record<string, string> = {
  file: "File",
  folder: "Folder",
  app: "Application",
  drive: "Drive",
};

const MAX_ATTEMPTS = 3;

export function UnlockWidget() {
  const [target, setTarget] = useState<UnlockTarget | null>(null);
  const [filterReq, setFilterReq] = useState<FilterAccessRequest | null>(null);
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);
  const [success, setSuccess] = useState(false);
  const [biometricAvailable, setBiometricAvailable] = useState(false);
  const [biometricLoading, setBiometricLoading] = useState(false);
  const [attempts, setAttempts] = useState(0);
  const [blocked, setBlocked] = useState(false);
  const [countdown, setCountdown] = useState(0);
  const countdownRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    const unlistenTarget = listen<UnlockTarget>("unlock-target", (event) => {
      setTarget(event.payload);
      setFilterReq(null);
      setPassword("");
      setError("");
      setSuccess(false);
      setAttempts(0);
      setBlocked(false);
      setCountdown(0);
      if (countdownRef.current) clearInterval(countdownRef.current);
    });

    const unlistenFilter = listen<FilterAccessRequest>("filter-access-request", (event) => {
      setFilterReq(event.payload);
      setTarget({
        target_type: "folder",
        target_id: event.payload.file_path,
        display_name: event.payload.display_name,
      });
      setPassword("");
      setError("");
      setSuccess(false);
      setAttempts(0);
      setBlocked(false);
      setCountdown(0);
      if (countdownRef.current) clearInterval(countdownRef.current);
    });

    checkBiometric().then(s => setBiometricAvailable(s.available)).catch(() => {});
    return () => {
      unlistenTarget.then(fn => fn());
      unlistenFilter.then(fn => fn());
      if (countdownRef.current) clearInterval(countdownRef.current);
    };
  }, []);

  const handleClose = async () => {
    if (filterReq) {
      await filterDeny(filterReq.message_id, filterReq.request_id).catch(() => {});
    }
    setTarget(null);
    setFilterReq(null);
    setPassword("");
    setError("");
    setSuccess(false);
    setAttempts(0);
    setBlocked(false);
    hideWidget();
  };

  const startLockout = () => {
    setBlocked(true);
    setCountdown(30);
    countdownRef.current = setInterval(() => {
      setCountdown(prev => {
        if (prev <= 1) {
          if (countdownRef.current) clearInterval(countdownRef.current);
          setBlocked(false);
          setAttempts(0);
          return 0;
        }
        return prev - 1;
      });
    }, 1000);
  };

  const handleSubmit = async () => {
    if (!password || blocked) return;
    setLoading(true);
    setError("");
    try {
      if (filterReq) {
        await filterUnlock(filterReq.message_id, filterReq.request_id, password);
        setSuccess(true);
        setTimeout(() => handleClose(), 1500);
      } else {
        await widgetUnlock(password);
        setSuccess(true);
        setTimeout(() => handleClose(), 1500);
      }
    } catch (e: any) {
      const newAttempts = attempts + 1;
      setAttempts(newAttempts);
      if (newAttempts >= MAX_ATTEMPTS) {
        setError(`Too many failed attempts. Blocked for 30 seconds.`);
        startLockout();
      } else {
        setError(`Incorrect password. ${MAX_ATTEMPTS - newAttempts} attempt(s) remaining.`);
      }
    }
    setLoading(false);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") handleSubmit();
    if (e.key === "Escape") handleClose();
  };

  const handleBiometric = async () => {
    if (blocked) return;
    setBiometricLoading(true);
    setError("");
    try {
      if (filterReq) {
        await widgetUnlockBiometric();
        setSuccess(true);
        setTimeout(() => handleClose(), 1500);
      } else {
        await widgetUnlockBiometric();
        setSuccess(true);
        setTimeout(() => handleClose(), 1500);
      }
    } catch (e: any) {
      const newAttempts = attempts + 1;
      setAttempts(newAttempts);
      if (newAttempts >= MAX_ATTEMPTS) {
        setError(`Too many failed attempts. Blocked for 30 seconds.`);
        startLockout();
      } else {
        setError(`Fingerprint failed. ${MAX_ATTEMPTS - newAttempts} attempt(s) remaining.`);
      }
    }
    setBiometricLoading(false);
  };

  return (
    <div className="min-h-screen flex items-center justify-center" style={{ background: "transparent" }}>
      <div className="w-[400px] rounded-2xl overflow-hidden shadow-2xl glass">
        <div data-tauri-drag-region
             className="h-10 w-full flex items-center justify-center cursor-move relative"
             style={{ background: "var(--gradient-brand)" }}>
          <Shield className="w-4 h-4 text-white mr-2" />
          <span className="text-xs font-medium text-white/90">InnologyLock</span>
          <button onClick={handleClose}
                  className="absolute right-2 top-1/2 -translate-y-1/2 p-1 rounded-md hover:bg-white/20 transition-colors">
            <X className="w-3.5 h-3.5 text-white/80" />
          </button>
        </div>

        <div className="p-6">
          {target && !success ? (
            <>
              <div className="flex items-center gap-3 mb-5">
                <div className="w-10 h-10 rounded-xl grid place-items-center"
                     style={{ background: "color-mix(in oklab, var(--cyan) 15%, transparent)" }}>
                  <Lock className="w-5 h-5 text-[color:var(--primary)]" />
                </div>
                <div>
                  <div className="text-sm font-medium">
                    {filterReq ? "Access Blocked" : "Unlock"} {typeLabels[target.target_type] || target.target_type}
                  </div>
                  <div className="text-xs text-[color:var(--muted-foreground)] truncate max-w-[260px]">{target.display_name}</div>
                </div>
              </div>

              {blocked ? (
                <div className="flex flex-col items-center py-4 gap-3">
                  <AlertTriangle className="w-8 h-8 text-[color:var(--destructive)]" />
                  <div className="text-sm font-medium text-[color:var(--destructive)]">Too Many Failed Attempts</div>
                  <div className="text-xs text-[color:var(--muted-foreground)]">
                    Wait {countdown} seconds before trying again
                  </div>
                  <div className="w-full h-2 rounded-full bg-surface overflow-hidden">
                    <div
                      className="h-full rounded-full transition-all duration-1000"
                      style={{
                        background: "var(--gradient-brand)",
                        width: `${(countdown / 30) * 100}%`,
                      }}
                    />
                  </div>
                  <button onClick={handleClose}
                          className="mt-2 px-6 py-2 rounded-lg text-sm bg-surface border border-surface-border hover:bg-surface-active">
                    Close
                  </button>
                </div>
              ) : (
                <>
                  <p className="text-xs text-[color:var(--muted-foreground)] mb-4">
                    {filterReq
                      ? "Enter your master password to access this locked folder."
                      : "Enter your master password to access this item."}
                  </p>

                  {error && (
                    <div className="mb-3 p-2.5 rounded-lg bg-[color:var(--destructive)]/15 border border-[color:var(--destructive)]/30 text-xs text-[color:var(--destructive)]">
                      {error}
                    </div>
                  )}

                  <input
                    type="password"
                    value={password}
                    onChange={e => setPassword(e.target.value)}
                    onKeyDown={handleKeyDown}
                    placeholder="Master password"
                    autoFocus
                    disabled={loading}
                    className="w-full px-4 py-3 rounded-lg bg-surface border border-surface-border text-sm outline-none focus:border-[color:var(--primary)]/50 mb-3 placeholder:text-[color:var(--muted-foreground)] disabled:opacity-50"
                  />

                  {biometricAvailable && (
                    <button
                      onClick={handleBiometric}
                      disabled={biometricLoading || loading}
                      className="w-full px-4 py-2.5 rounded-lg text-sm font-medium mb-4 flex items-center justify-center gap-2 border border-[color:var(--accent)]/30 hover:bg-[color:var(--accent)]/10 transition-colors disabled:opacity-40"
                      style={{ color: "var(--accent)" }}>
                      {biometricLoading ? (
                        "Verifying fingerprint..."
                      ) : (
                        <>
                          <Fingerprint className="w-4 h-4" />
                          Use Fingerprint
                        </>
                      )}
                    </button>
                  )}

                  {biometricAvailable && (
                    <div className="flex items-center gap-3 mb-4">
                      <div className="flex-1 h-px bg-surface-border"></div>
                      <span className="text-[10px] uppercase tracking-widest text-[color:var(--muted-foreground)]">or</span>
                      <div className="flex-1 h-px bg-surface-border"></div>
                    </div>
                  )}

                  <div className="flex gap-2">
                    <button onClick={handleClose}
                            className="px-4 py-2.5 rounded-lg text-sm bg-surface border border-surface-border hover:bg-surface-active flex-1">
                      Cancel
                    </button>
                    <button
                      onClick={handleSubmit}
                      disabled={!password || loading}
                      className="flex-[2] px-4 py-2.5 rounded-lg text-sm font-medium text-[color:var(--primary-foreground)] flex items-center justify-center gap-2 glow-cyan disabled:opacity-40"
                      style={{ background: "var(--gradient-brand)" }}>
                      {loading ? "Verifying..." : "Unlock"} <Lock className="w-4 h-4" />
                    </button>
                  </div>

                  {attempts > 0 && !blocked && (
                    <div className="mt-3 text-center text-[10px] text-[color:var(--muted-foreground)]">
                      {attempts}/{MAX_ATTEMPTS} failed attempts
                    </div>
                  )}
                </>
              )}
            </>
          ) : success ? (
            <div className="text-center py-4">
              <div className="w-12 h-12 mx-auto mb-3 rounded-full grid place-items-center" style={{ background: "color-mix(in oklab, var(--success) 15%, transparent)" }}>
                <Shield className="w-6 h-6 text-[color:var(--success)]" />
              </div>
              <div className="text-sm font-medium text-[color:var(--success)]">Unlocked Successfully</div>
              <div className="text-xs text-[color:var(--muted-foreground)] mt-1">
                {filterReq ? "Folder is now accessible." : target?.target_type === "app" ? "Launching application..." : "You can now access this item."}
              </div>
            </div>
          ) : (
            <div className="text-center py-6 text-sm text-[color:var(--muted-foreground)]">
              No locked items to unlock.
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
