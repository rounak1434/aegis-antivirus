//! Embed an `asInvoker` application manifest into the MSVC binaries (incl. test
//! and bench executables). Without this, Windows UAC installer-detection
//! auto-elevates any executable whose name contains "update" — so the crate's
//! test binary (`aegis_update-*.exe`) fails to launch with "requires elevation"
//! (os error 740). The manifest opts out of that heuristic.

fn main() {
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc") {
        let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let manifest = format!("{dir}\\app.manifest");
        println!("cargo:rerun-if-changed=app.manifest");
        // Plain rustc-link-arg covers every linked artifact of this crate
        // (tests, benches, the lib unit-test harness) — all need the manifest.
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTINPUT:{manifest}");
    }
}
