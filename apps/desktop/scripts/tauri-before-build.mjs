import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";

const desktopRoot = resolve(fileURLToPath(new URL("..", import.meta.url)));
const tauriSidecarDir = resolve(desktopRoot, "src-tauri/sidecars");
const prepareSidecarScript = resolve(desktopRoot, "scripts/prepare-sidecar.mjs");

const pnpmCmd = process.platform === "win32" ? "corepack.cmd" : "pnpm";
const pnpmArgs = process.platform === "win32" ? ["pnpm"] : [];

const runPnpm = (args) => {
  const result = spawnSync(pnpmCmd, [...pnpmArgs, ...args], {
    stdio: "inherit",
    shell: process.platform === "win32",
  });

  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
};

runPnpm(["-C", "../..", "--filter", "openwork-server", "build:bin"]);

const prepareSidecar = spawnSync(process.execPath, [
  prepareSidecarScript,
  "--force",
  "--outdir",
  tauriSidecarDir,
], {
  cwd: desktopRoot,
  stdio: "inherit",
});
if (prepareSidecar.status !== 0) {
  process.exit(prepareSidecar.status ?? 1);
}

runPnpm(["--filter", "@openwork/app", "build"]);
