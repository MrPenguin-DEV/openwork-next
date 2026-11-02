// Runtime detection primitives. Leaf module by design: keep it import-free so
// low-level clients (opencode, openwork-server, den) can use it without
// dragging in the utils barrel (which pulls i18n and app constants).
export function isElectronRuntime() {
  return typeof window !== "undefined" && (window as Window).__OPENWORK_ELECTRON__ != null;
}

export function isTauriRuntime() {
  return typeof window !== "undefined" && Reflect.get(window, "__TAURI_INTERNALS__") != null;
}

export function isDesktopRuntime() {
  return isElectronRuntime() || isTauriRuntime();
}
