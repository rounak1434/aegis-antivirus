// The service runtime + orchestrator live in the `aegis_service` library; this
// binary is the thin Windows-service host that drives them.
#[cfg(windows)]
mod windows_host;

#[cfg(windows)]
fn main() -> windows_service::Result<()> {
    windows_host::run_service_dispatcher()
}

#[cfg(not(windows))]
fn main() {
    eprintln!("AegisService is a Windows service and must be built on Windows.");
}
