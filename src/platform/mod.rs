//! Platform abstraction layer

/// Open a URI via the system default handler.
/// P0: currently returns error if no handler found.
pub fn open_uri(uri: &str) -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    {
        // On Windows, use cmd /c start to open via default handler
        // The empty string before the URI is the window title parameter
        std::process::Command::new("cmd")
            .args(["/c", "start", "", uri])
            .spawn()
            .map_err(|e| anyhow::anyhow!("failed to open URI '{}': {}", uri, e))?;
        Ok(())
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(uri)
            .spawn()
            .map_err(|e| anyhow::anyhow!("failed to open URI '{}': {}", uri, e))?;
        Ok(())
    }

    #[cfg(target_os = "linux")]
    {
        // Try xdg-open, fall back to sensible-browser
        let result = std::process::Command::new("xdg-open")
            .arg(uri)
            .spawn();

        match result {
            Ok(_) => Ok(()),
            Err(_) => {
                // Fallback to sensible-browser
                std::process::Command::new("sensible-browser")
                    .arg(uri)
                    .spawn()
                    .map_err(|e| anyhow::anyhow!("failed to open URI '{}': {}", uri, e))?;
                Ok(())
            }
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        Err(anyhow::anyhow!("open_uri not supported on this platform"))
    }
}

