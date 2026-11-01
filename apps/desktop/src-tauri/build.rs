use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

fn main() {
    emit_build_info();
    emit_desktop_bootstrap_seed();
    ensure_opencode_sidecar();
    ensure_openwork_server_sidecar();
    ensure_orchestrator_sidecar();
    ensure_chrome_devtools_mcp_sidecar();
    ensure_versions_manifest();
    tauri_build::build();
}

fn ensure_chrome_devtools_mcp_sidecar() {
    let target = env::var("CARGO_CFG_TARGET_TRIPLE")
        .or_else(|_| env::var("TARGET"))
        .or_else(|_| env::var("TAURI_ENV_TARGET_TRIPLE"))
        .unwrap_or_default();
    if target.is_empty() {
        return;
    }

    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    let sidecar_dir = manifest_dir.join("sidecars");

    let canonical_name = if target.contains("windows") {
        "chrome-devtools-mcp.exe"
    } else {
        "chrome-devtools-mcp"
    };

    let mut target_name = format!("chrome-devtools-mcp-{target}");
    if target.contains("windows") {
        target_name.push_str(".exe");
    }

    let dest_path = sidecar_dir.join(canonical_name);
    let target_dest_path = sidecar_dir.join(target_name);

    let profile = env::var("PROFILE").unwrap_or_default();
    if !dest_path.exists() {
        create_debug_stub(&dest_path, &sidecar_dir, &profile, &target);
    }
    if !target_dest_path.exists() {
        create_debug_stub(&target_dest_path, &sidecar_dir, &profile, &target);
    }
}

fn ensure_versions_manifest() {
    let profile = env::var("PROFILE").unwrap_or_default();
    if profile == "release" {
        return;
    }

    let target = env::var("CARGO_CFG_TARGET_TRIPLE")
        .or_else(|_| env::var("TARGET"))
        .or_else(|_| env::var("TAURI_ENV_TARGET_TRIPLE"))
        .unwrap_or_default();
    if target.is_empty() {
        return;
    }

    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    let sidecar_dir = manifest_dir.join("sidecars");
    let canonical_path = sidecar_dir.join("versions.json");
    let target_path = sidecar_dir.join(format!("versions.json-{target}"));

    if canonical_path.exists() && target_path.exists() {
        return;
    }

    if fs::create_dir_all(&sidecar_dir).is_err() {
        return;
    }

    if !canonical_path.exists() {
        let _ = fs::write(&canonical_path, "{}\n");
    }
    if !target_path.exists() {
        let _ = fs::write(&target_path, "{}\n");
    }
}

fn emit_build_info() {
    let sha = env::var("OPENWORK_GIT_SHA")
        .ok()
        .and_then(|value| {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        })
        .or_else(|| {
            let output = Command::new("git")
                .args(["rev-parse", "HEAD"])
                .output()
                .ok()?;
            if !output.status.success() {
                return None;
            }
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if value.is_empty() {
                None
            } else {
                Some(value)
            }
        });
    if let Some(value) = sha {
        println!("cargo:rustc-env=OPENWORK_GIT_SHA={}", value);
    }

    let build_epoch = env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|value| {
            let trimmed = value.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        })
        .or_else(|| {
            // Best-effort fallback; not reproducible, but useful in dev builds.
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_secs();
            Some(now.to_string())
        });

    if let Some(value) = build_epoch {
        println!("cargo:rustc-env=OPENWORK_BUILD_EPOCH={}", value);
    }
}

fn emit_desktop_bootstrap_seed() {
    for key in [
        "OPENWORK_DESKTOP_DEN_BASE_URL",
        "OPENWORK_DESKTOP_DEN_API_BASE_URL",
        "OPENWORK_DESKTOP_DEN_REQUIRE_SIGNIN",
        "VITE_DEN_BASE_URL",
        "VITE_DEN_API_BASE_URL",
        "VITE_DEN_REQUIRE_SIGNIN",
    ] {
        println!("cargo:rerun-if-env-changed={key}");
    }

    if let Some(value) = env::var("OPENWORK_DESKTOP_DEN_BASE_URL")
        .ok()
        .or_else(|| env::var("VITE_DEN_BASE_URL").ok())
        .map(|entry| entry.trim().to_string())
        .filter(|entry| !entry.is_empty())
    {
        println!("cargo:rustc-env=OPENWORK_DESKTOP_DEN_BASE_URL={value}");
    }

    if let Some(value) = env::var("OPENWORK_DESKTOP_DEN_API_BASE_URL")
        .ok()
        .or_else(|| env::var("VITE_DEN_API_BASE_URL").ok())
        .map(|entry| entry.trim().to_string())
        .filter(|entry| !entry.is_empty())
    {
        println!("cargo:rustc-env=OPENWORK_DESKTOP_DEN_API_BASE_URL={value}");
    }

    if let Some(value) = env::var("OPENWORK_DESKTOP_DEN_REQUIRE_SIGNIN")
        .ok()
        .or_else(|| env::var("VITE_DEN_REQUIRE_SIGNIN").ok())
        .map(|entry| entry.trim().to_string())
        .filter(|entry| !entry.is_empty())
    {
        println!("cargo:rustc-env=OPENWORK_DESKTOP_DEN_REQUIRE_SIGNIN={value}");
    }
}

fn ensure_orchestrator_sidecar() {
    let target = env::var("CARGO_CFG_TARGET_TRIPLE")
        .or_else(|_| env::var("TARGET"))
        .or_else(|_| env::var("TAURI_ENV_TARGET_TRIPLE"))
        .unwrap_or_default();
    if target.is_empty() {
        return;
    }

    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    let sidecar_dir = manifest_dir.join("sidecars");

    let canonical_name = if target.contains("windows") {
        "openwork-orchestrator.exe"
    } else {
        "openwork-orchestrator"
    };
    let mut target_name = format!("openwork-orchestrator-{target}");
    if target.contains("windows") {
        target_name.push_str(".exe");
    }

    let dest_path = sidecar_dir.join(canonical_name);
    let target_dest_path = sidecar_dir.join(target_name);

    let profile = env::var("PROFILE").unwrap_or_default();
    if !dest_path.exists() {
        create_debug_stub(&dest_path, &sidecar_dir, &profile, &target);
    }
    if !target_dest_path.exists() {
        create_debug_stub(&target_dest_path, &sidecar_dir, &profile, &target);
    }

    if dest_path.exists() && target_dest_path.exists() {
        return;
    }

    if target_dest_path.exists() && !dest_path.exists() {
        if copy_sidecar(&target_dest_path, &dest_path, &target) {
            return;
        }
    }

    let source_path = env::var("OPENWORK_ORCHESTRATOR_BIN_PATH")
        .ok()
        .map(PathBuf::from)
        .filter(|path| path.is_file())
        .or_else(|| {
            find_in_path(if target.contains("windows") {
                "openwork.exe"
            } else {
                "openwork"
            })
        });

    let Some(source_path) = source_path else {
        println!(
            "cargo:warning=orchestrator sidecar missing at {} (set OPENWORK_ORCHESTRATOR_BIN_PATH or install openwork-orchestrator)",
            dest_path.display()
        );
        create_debug_stub(&dest_path, &sidecar_dir, &profile, &target);
        create_debug_stub(&target_dest_path, &sidecar_dir, &profile, &target);
        return;
    };

    if fs::create_dir_all(&sidecar_dir).is_err() {
        return;
    }

    let copied = copy_sidecar(&source_path, &dest_path, &target);
    if copied {
        #[cfg(unix)]
        {
            let _ = fs::set_permissions(&dest_path, fs::Permissions::from_mode(0o755));
        }
        let _ = copy_sidecar(&dest_path, &target_dest_path, &target);
    } else {
        println!(
            "cargo:warning=Failed to copy orchestrator sidecar from {} to {}",
            source_path.display(),
            dest_path.display()
        );
        create_debug_stub(&dest_path, &sidecar_dir, &profile, &target);
        create_debug_stub(&target_dest_path, &sidecar_dir, &profile, &target);
    }

    if !dest_path.exists() {
        create_debug_stub(&dest_path, &sidecar_dir, &profile, &target);
    }
    if !target_dest_path.exists() {
        create_debug_stub(&target_dest_path, &sidecar_dir, &profile, &target);
    }
}

fn is_debug_stub(path: &PathBuf) -> bool {
    fs::read(path)
        .map(|bytes| bytes.starts_with(b"#!/usr/bin/env bash"))
        .unwrap_or(false)
}

fn ensure_opencode_sidecar() {
    let target = env::var("CARGO_CFG_TARGET_TRIPLE")
        .or_else(|_| env::var("TARGET"))
        .or_else(|_| env::var("TAURI_ENV_TARGET_TRIPLE"))
        .unwrap_or_default();
    if target.is_empty() {
        return;
    }

    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    let sidecar_dir = manifest_dir.join("sidecars");
    let canonical_name = if target.contains("windows") { "opencode.exe" } else { "opencode" };
    let mut target_name = format!("opencode-{target}");
    if target.contains("windows") {
        target_name.push_str(".exe");
    }

    let dest_path = sidecar_dir.join(canonical_name);
    let target_dest_path = sidecar_dir.join(target_name);
    if dest_path.is_file() && !is_debug_stub(&dest_path) {
        if !target_dest_path.is_file() || is_debug_stub(&target_dest_path) {
            let _ = copy_sidecar(&dest_path, &target_dest_path, &target);
        }
        return;
    }
    if target_dest_path.is_file() && !is_debug_stub(&target_dest_path) {
        if copy_sidecar(&target_dest_path, &dest_path, &target) {
            return;
        }
    }

    // prepare-sidecar.mjs stages the current OpenCode binary here before the
    // Tauri command starts. Prefer this exact artifact over PATH.
    let resource_path = manifest_dir
        .join("../../resources/sidecars")
        .join(canonical_name);
    let source_path = env::var("OPENCODE_BIN_PATH")
        .ok()
        .map(PathBuf::from)
        .filter(|path| path.is_file() && !is_debug_stub(path))
        .or_else(|| resource_path.is_file().then_some(resource_path))
        .or_else(|| {
            find_in_path(canonical_name).filter(|path| path.is_file() && !is_debug_stub(path))
        });

    let profile = env::var("PROFILE").unwrap_or_default();
    let Some(source_path) = source_path else {
        println!(
            "cargo:warning=OpenCode sidecar missing at {} (set OPENCODE_BIN_PATH or install OpenCode)",
            dest_path.display()
        );
        create_debug_stub(&dest_path, &sidecar_dir, &profile, &target);
        create_debug_stub(&target_dest_path, &sidecar_dir, &profile, &target);
        return;
    };

    if fs::create_dir_all(&sidecar_dir).is_err() {
        return;
    }
    if copy_sidecar(&source_path, &dest_path, &target) {
        #[cfg(unix)]
        {
            let _ = fs::set_permissions(&dest_path, fs::Permissions::from_mode(0o755));
        }
        let _ = copy_sidecar(&dest_path, &target_dest_path, &target);
    } else {
        println!(
            "cargo:warning=Failed to copy OpenCode sidecar from {} to {}",
            source_path.display(),
            dest_path.display()
        );
        create_debug_stub(&dest_path, &sidecar_dir, &profile, &target);
        create_debug_stub(&target_dest_path, &sidecar_dir, &profile, &target);
    }
}

fn ensure_openwork_server_sidecar() {
    let target = env::var("CARGO_CFG_TARGET_TRIPLE")
        .or_else(|_| env::var("TARGET"))
        .or_else(|_| env::var("TAURI_ENV_TARGET_TRIPLE"))
        .unwrap_or_default();
    if target.is_empty() {
        return;
    }

    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));
    let sidecar_dir = manifest_dir.join("sidecars");
    let canonical_name = if target.contains("windows") {
        "openwork-server.exe"
    } else {
        "openwork-server"
    };
    let mut target_name = format!("openwork-server-{target}");
    if target.contains("windows") {
        target_name.push_str(".exe");
    }

    let dest_path = sidecar_dir.join(canonical_name);
    let target_dest_path = sidecar_dir.join(target_name);
    if dest_path.is_file() && !is_debug_stub(&dest_path) {
        if !target_dest_path.is_file() || is_debug_stub(&target_dest_path) {
            let _ = copy_sidecar(&dest_path, &target_dest_path, &target);
        }
        return;
    }
    if target_dest_path.is_file() && !is_debug_stub(&target_dest_path) {
        if copy_sidecar(&target_dest_path, &dest_path, &target) {
            return;
        }
    }

    // Built by tauri-before-dev.mjs / tauri-before-build.mjs from the current
    // apps/server CLI. This keeps the Rust host on the same server version.
    let workspace_server_path = manifest_dir
        .join("../../server/dist/bin")
        .join(canonical_name);
    let source_path = env::var("OPENWORK_SERVER_BIN_PATH")
        .ok()
        .map(PathBuf::from)
        .filter(|path| path.is_file() && !is_debug_stub(path))
        .or_else(|| workspace_server_path.is_file().then_some(workspace_server_path))
        .or_else(|| {
            find_in_path(canonical_name).filter(|path| path.is_file() && !is_debug_stub(path))
        });

    let profile = env::var("PROFILE").unwrap_or_default();
    let Some(source_path) = source_path else {
        println!(
            "cargo:warning=OpenWork server sidecar missing at {} (set OPENWORK_SERVER_BIN_PATH or build openwork-server)",
            dest_path.display()
        );
        create_debug_stub(&dest_path, &sidecar_dir, &profile, &target);
        create_debug_stub(&target_dest_path, &sidecar_dir, &profile, &target);
        return;
    };

    if fs::create_dir_all(&sidecar_dir).is_err() {
        return;
    }
    if copy_sidecar(&source_path, &dest_path, &target) {
        #[cfg(unix)]
        {
            let _ = fs::set_permissions(&dest_path, fs::Permissions::from_mode(0o755));
        }
        let _ = copy_sidecar(&dest_path, &target_dest_path, &target);
    } else {
        println!(
            "cargo:warning=Failed to copy OpenWork server sidecar from {} to {}",
            source_path.display(),
            dest_path.display()
        );
        create_debug_stub(&dest_path, &sidecar_dir, &profile, &target);
        create_debug_stub(&target_dest_path, &sidecar_dir, &profile, &target);
    }
}

fn copy_sidecar(source_path: &PathBuf, dest_path: &PathBuf, target: &str) -> bool {
    let mut copied = fs::copy(source_path, dest_path).is_ok();

    #[cfg(unix)]
    if !copied {
        if std::os::unix::fs::symlink(source_path, dest_path).is_ok() {
            copied = true;
        }
    }

    #[cfg(windows)]
    if !copied {
        if fs::hard_link(source_path, dest_path).is_ok() {
            copied = true;
        }
    }

    if copied {
        #[cfg(unix)]
        {
            let _ = fs::set_permissions(dest_path, fs::Permissions::from_mode(0o755));
        }
    } else if target.contains("windows") {
        let _ = fs::remove_file(dest_path);
    }

    copied
}

fn find_in_path(binary: &str) -> Option<PathBuf> {
    let paths = env::var_os("PATH")?;
    env::split_paths(&paths).find_map(|dir| {
        let candidate = dir.join(binary);
        if candidate.is_file() {
            Some(candidate)
        } else {
            None
        }
    })
}

fn create_debug_stub(dest_path: &PathBuf, sidecar_dir: &PathBuf, profile: &str, target: &str) {
    if profile == "release" || target.contains("windows") {
        return;
    }

    if fs::create_dir_all(sidecar_dir).is_err() {
        return;
    }

    let stub = "#!/usr/bin/env bash\n\
echo 'Sidecar missing. Install the binary or set the *_BIN_PATH env var.'\n\
exit 1\n";
    if fs::write(dest_path, stub).is_ok() {
        #[cfg(unix)]
        let _ = fs::set_permissions(dest_path, fs::Permissions::from_mode(0o755));
    }
}
