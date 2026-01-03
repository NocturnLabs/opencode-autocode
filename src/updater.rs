use anyhow::{Context, Result};
use console::style;
use self_update::cargo_crate_version;

/// Check for updates and print a notification if one is available.
/// Returns Ok(Some(version)) if update available, Ok(None) otherwise.
pub fn check_for_update() -> Result<Option<String>> {
    let current_version = cargo_crate_version!();

    // Check for a new release
    // We suppress errors here because we don't want to crash the app if internet is down
    // or GitHub is rate limiting.
    let releases = match self_update::backends::github::ReleaseList::configure()
        .repo_owner("NocturnLabs")
        .repo_name("opencode-autocode")
        .build()
        .and_then(|r| r.fetch())
    {
        Ok(r) => r,
        Err(_) => return Ok(None),
    };

    if let Some(latest) = releases.first() {
        let is_greater = self_update::version::bump_is_greater(current_version, &latest.version)
            .unwrap_or(false);
        if is_greater {
            return Ok(Some(latest.version.clone()));
        }
    }

    Ok(None)
}

/// Perform the self-update process.
pub fn update() -> Result<()> {
    let current_version = cargo_crate_version!();
    println!("Checking for updates... (Current: {})", current_version);

    let status = self_update::backends::github::Update::configure()
        .repo_owner("NocturnLabs")
        .repo_name("opencode-autocode")
        .bin_name("opencode-autocode")
        .bin_path_in_archive("opencode-autocode")
        .show_download_progress(true)
        .current_version(current_version)
        .build()
        .context("Failed to build updater configuration")?
        .update()
        .context("Failed to update binary")?;

    if status.updated() {
        println!(
            "{}",
            style(format!(
                "✅ Successfully updated to version {}!",
                status.version()
            ))
            .green()
            .bold()
        );
    } else {
        println!("{}", style("Already up to date.").green());
    }

    Ok(())
}
