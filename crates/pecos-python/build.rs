use std::env;
use std::process::Command;

fn main() {
    // Detect if we're building with Maturin
    let is_maturin_build = env::var("PYO3_PYTHON").is_ok();

    // Configure PyO3 build settings
    pyo3_build_config::add_extension_module_link_args();

    if is_maturin_build {
        // If Maturin is running, do not set custom paths, rely on Maturin's configuration
        println!("cargo:rerun-if-env-changed=PYO3_PYTHON");
        println!("cargo:rerun-if-env-changed=PYTHON_SYS_EXECUTABLE");
        println!("cargo:rerun-if-env-changed=PATH");
    } else {
        // For regular cargo builds (e.g., cargo test), set Python library paths
        let python_executable = env::var("PYTHON_SYS_EXECUTABLE").unwrap_or_else(|_| "python3".to_string());

        let python_libdir_output = Command::new(&python_executable)
            .arg("-c")
            .arg("import sysconfig; print(sysconfig.get_config_var('LIBDIR'))")
            .output()
            .expect("Failed to query Python LIBDIR. Ensure Python is installed and accessible.");
        let libdir = String::from_utf8(python_libdir_output.stdout)
            .expect("Invalid UTF-8 in Python LIBDIR output")
            .trim()
            .to_string();

        let python_ldversion_output = Command::new(&python_executable)
            .arg("-c")
            .arg("import sysconfig; print(sysconfig.get_config_var('LDVERSION') or sysconfig.get_config_var('VERSION'))")
            .output()
            .expect("Failed to query Python LDVERSION. Ensure Python is installed and accessible.");
        let ldversion = String::from_utf8(python_ldversion_output.stdout)
            .expect("Invalid UTF-8 in Python LDVERSION output")
            .trim()
            .to_string();

        println!("cargo:rustc-link-search=native={}", libdir);

        #[cfg(target_os = "windows")]
        println!("cargo:rustc-link-lib=dylib=python{}", ldversion);

        #[cfg(not(target_os = "windows"))]
        println!("cargo:rustc-link-lib=python{}", ldversion);

        println!("cargo:rerun-if-env-changed=PYTHON_SYS_EXECUTABLE");
        println!("cargo:rerun-if-env-changed=PYTHONHOME");
        println!("cargo:rerun-if-env-changed=PATH");
    }
}
