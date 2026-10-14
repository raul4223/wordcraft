//! Show a file in the system file manager (File › Home › Show in Finder).

pub fn reveal(path: &str) {
    let p = std::path::Path::new(path);
    #[cfg(target_os = "macos")]
    let r = std::process::Command::new("open").arg("-R").arg(p).spawn().map(|_| ());
    #[cfg(target_os = "windows")]
    let r = std::process::Command::new("explorer").arg(format!("/select,{}", p.display())).spawn().map(|_| ());
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let r = std::process::Command::new("xdg-open").arg(p.parent().unwrap_or(p)).spawn().map(|_| ());
    if let Err(e) = r {
        log::warn!("reveal {path}: {e}");
    }
}
