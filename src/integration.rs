//! Integration with the Linux desktop: install/uninstall XDG desktop entry
//! and icon so the application appears in the system application menu.

use anyhow::{Context, Result, anyhow};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// 256×256 PNG icon, embedded at compile time.
pub const ICON_256_BYTES: &[u8] = include_bytes!("../data/icon256.png");
/// 48×48 PNG icon, embedded at compile time.
pub const ICON_48_BYTES: &[u8] = include_bytes!("../data/icon48.png");
/// XDG desktop entry template with an `{EXEC}` placeholder for the binary path.
const DESKTOP_TEMPLATE: &str = include_str!("../data/barafu-black-curtain.desktop.template");

/// Returns either the bare binary name (if it is in `$PATH` and resolves to the
/// same file as `current_exe`) or the absolute path to the current executable.
/// This lets the `.desktop` file use a simple name when installed system-wide.
fn determine_executable() -> Result<String> {
    let current_exe = env::current_exe()
        .context("Failed to get current executable path")?
        .canonicalize()
        .context("Failed to canonicalize current executable path")?;

    let binary_name = current_exe
        .file_name()
        .ok_or_else(|| anyhow!("Failed to get filename from current executable path"))?
        .to_string_lossy()
        .into_owned();

    // Check whether the bare name is on PATH → same file
    let type_command = format!("type -p {}", &binary_name);
    let which_output = Command::new("sh")
        .args(["-c", &type_command])
        .output()
        .with_context(|| format!("Failed to run 'type -p {}'", binary_name))?;

    if which_output.status.success() {
        let path_str = String::from_utf8_lossy(&which_output.stdout)
            .trim()
            .to_string();
        if let Ok(which_path) = PathBuf::from(&path_str).canonicalize()
            && which_path == current_exe
        {
            return Ok(binary_name);
        }
    }

    // Fall back to the absolute path
    Ok(current_exe.to_string_lossy().into_owned())
}

/// Writes `content` to a temporary file named `filename` and returns its path.
fn create_temp_file(content: &[u8], filename: &str) -> Result<PathBuf> {
    let temp_dir = env::temp_dir();
    let file_path = temp_dir.join(filename);
    fs::write(&file_path, content)
        .with_context(|| format!("Failed to write temporary file {}", file_path.display()))?;
    Ok(file_path)
}

/// Registers the application in the XDG system menu by installing a desktop
/// entry and a icon via `xdg-desktop-menu` and `xdg-icon-resource`.
pub fn install() -> Result<()> {
    eprintln!("Installing application to system menu");

    // Fill in the executable path in the desktop template
    let exec = determine_executable()?;
    let desktop_content = DESKTOP_TEMPLATE.replace("{EXEC}", &exec);

    let desktop_temp = create_temp_file(desktop_content.as_bytes(), "barafu-black-curtain.desktop")?;

    let status = Command::new("xdg-desktop-menu")
        .arg("install")
        .arg(&desktop_temp)
        .status()
        .context("Failed to execute xdg-desktop-menu")?;

    if !status.success() {
        return Err(anyhow!(
            "xdg-desktop-menu failed with exit code {:?}",
            status.code()
        ));
    }

    // Install the icon resources
    for (icon_bytes, size) in [
        (ICON_256_BYTES, "256"),
        (ICON_48_BYTES, "48"),
    ] {
        let icon_temp = create_temp_file(icon_bytes, &format!("barafu-black-curtain-{}.png", size))?;

        let status = Command::new("xdg-icon-resource")
            .args(["install", "--size", size, "--context", "apps"])
            .arg(&icon_temp)
            .arg("barafu-black-curtain")
            .status()
            .with_context(|| format!("Failed to execute xdg-icon-resource install --size {}", size))?;

        if !status.success() {
            return Err(anyhow!(
                "xdg-icon-resource install --size {} failed with exit code {:?}",
                size,
                status.code()
            ));
        }
    }

    eprintln!("Installation completed successfully");
    Ok(())
}

/// Removes the application from the XDG system menu.
pub fn uninstall() -> Result<()> {
    eprintln!("Removing application from system menu");

    // Re-generate the identical desktop file (needed for the uninstall command)
    let exec = determine_executable()?;
    let desktop_content = DESKTOP_TEMPLATE.replace("{EXEC}", &exec);
    let desktop_temp = create_temp_file(desktop_content.as_bytes(), "barafu-black-curtain.desktop")?;

    let status = Command::new("xdg-desktop-menu")
        .arg("uninstall")
        .arg(&desktop_temp)
        .status()
        .context("Failed to execute xdg-desktop-menu uninstall")?;

    if !status.success() {
        return Err(anyhow!(
            "xdg-desktop-menu uninstall failed with exit code {:?}",
            status.code()
        ));
    }

    for size in ["256", "48"] {
        let status = Command::new("xdg-icon-resource")
            .args(["uninstall", "--size", size, "--context", "apps"])
            .arg("barafu-black-curtain")
            .status()
            .with_context(|| format!("Failed to execute xdg-icon-resource uninstall --size {}", size))?;

        if !status.success() {
            return Err(anyhow!(
                "xdg-icon-resource uninstall --size {} failed with exit code {:?}",
                size,
                status.code()
            ));
        }
    }

    eprintln!("Uninstallation completed successfully");
    Ok(())
}
