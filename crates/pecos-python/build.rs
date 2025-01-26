use std::env;
use std::path::PathBuf;
use std::process::Command;
use serde_json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure PyO3 build settings
    pyo3_build_config::add_extension_module_link_args();

    // Detect if we're building with Maturin
    if env::var("PYO3_PYTHON").is_ok() {
        eprintln!("Running under Maturin build");
        println!("cargo:rerun-if-env-changed=PYO3_PYTHON");
        println!("cargo:rerun-if-env-changed=PYTHON_SYS_EXECUTABLE");
        println!("cargo:rerun-if-changed=PATH");
        return Ok(());
    }

    // Find Python executable
    let python_executable = env::var("PYTHON_SYS_EXECUTABLE")
        .or_else(|_| env::var("UV_PYTHON"))
        .unwrap_or_else(|_| {
            let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("Failed to get CARGO_MANIFEST_DIR");
            let venv_python = PathBuf::from(manifest_dir)
                .parent()
                .and_then(|p| p.parent())
                .map(|p| p.join(".venv").join("bin").join("python"))
                .expect("Failed to construct venv path")
                .to_string_lossy()
                .to_string();

            if std::path::Path::new(&venv_python).exists() {
                venv_python
            } else {
                "python3".to_string()
            }
        });

    eprintln!("Using Python executable: {}", python_executable);

    // Get Python configuration and paths
    let python_info = Command::new(&python_executable)
        .arg("-c")
        .arg(r#"
import sys
import os
import json
import sysconfig
import glob
from pathlib import Path

def find_lib_files(directory, pattern="libpython*"):
    try:
        return list(str(p) for p in Path(directory).glob(pattern))
    except Exception as e:
        print(f"Error scanning {directory}: {e}", file=sys.stderr)
        return []

info = {
    'executable': sys.executable,
    'prefix': sys.prefix,
    'realpath': os.path.realpath(sys.executable),
    'version': sys.version.split()[0],
    'version_info': {
        'major': sys.version_info.major,
        'minor': sys.version_info.minor,
        'micro': sys.version_info.micro,
    },
    'config_vars': {
        'LDVERSION': sysconfig.get_config_var('LDVERSION'),
        'INSTSONAME': sysconfig.get_config_var('INSTSONAME'),
        'BLDLIBRARY': sysconfig.get_config_var('BLDLIBRARY'),
        'LIBDIR': sysconfig.get_config_var('LIBDIR'),
        'PY_ENABLE_SHARED': sysconfig.get_config_var('PY_ENABLE_SHARED'),
        'LIBRARY': sysconfig.get_config_var('LIBRARY'),
        'LDLIBRARY': sysconfig.get_config_var('LDLIBRARY'),
    }
}

# Find UV installation
info['uv'] = {
    'root': os.path.join(os.path.expanduser('~'), '.local', 'share', 'uv'),
    'venv': os.path.dirname(os.path.dirname(sys.executable)),
}

version_str = f'{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}'
platform_str = 'linux-x86_64-gnu'  # TODO: Make this dynamic
info['uv']['lib_path'] = os.path.join(
    info['uv']['root'],
    'python',
    f'cpython-{version_str}-{platform_str}',
    'lib'
)

# List all potential library locations
lib_paths = [
    info['uv']['lib_path'],
    os.path.join(info['uv']['venv'], 'lib'),
    '/usr/lib',
    '/usr/local/lib',
    '/usr/lib/x86_64-linux-gnu',
]

# Add Python-specific paths
lib_paths.extend([
    os.path.join(path, f'python{sys.version_info.major}.{sys.version_info.minor}')
    for path in lib_paths
])

info['lib_files'] = {}
for path in lib_paths:
    if os.path.exists(path):
        info['lib_files'][path] = find_lib_files(path)
        # Also look in lib-dynload
        dynload = os.path.join(path, 'lib-dynload')
        if os.path.exists(dynload):
            info['lib_files'][dynload] = find_lib_files(dynload)

print(json.dumps(info))
"#)
        .output()?;

    if !python_info.stderr.is_empty() {
        eprintln!("Python stderr: {}", String::from_utf8_lossy(&python_info.stderr));
    }

    let python_info: serde_json::Value = serde_json::from_slice(&python_info.stdout)?;
    eprintln!("Python info: {}", serde_json::to_string_pretty(&python_info)?);

    // Collect all library paths
    let mut lib_paths = vec![
        PathBuf::from(python_info["uv"]["lib_path"].as_str().unwrap()),
        PathBuf::from(python_info["uv"]["venv"].as_str().unwrap()).join("lib"),
        PathBuf::from("/usr/lib"),
        PathBuf::from("/usr/local/lib"),
        PathBuf::from("/usr/lib/x86_64-linux-gnu"),
    ];

    // Add library search paths and rpaths
    for path in &lib_paths {
        if path.exists() {
            eprintln!("Adding library path: {}", path.display());
            println!("cargo:rustc-link-search=native={}", path.display());
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", path.display());
        }
    }

    // Add Python-specific paths
    let version = format!("{}.{}",
                          python_info["version_info"]["major"].as_i64().unwrap(),
                          python_info["version_info"]["minor"].as_i64().unwrap()
    );
    let version_path = format!("python{}", version);

    for path in lib_paths.clone() {
        let python_lib_path = path.join(&version_path);
        if python_lib_path.exists() {
            lib_paths.push(python_lib_path);
        }
    }

    // Try LDLIBRARY first
    if let Some(ldlibrary) = python_info["config_vars"]["LDLIBRARY"].as_str() {
        eprintln!("Found LDLIBRARY: {}", ldlibrary);
        if let Some(libname) = ldlibrary.strip_prefix("lib").and_then(|s| s.strip_suffix(".so")) {
            eprintln!("Using library name: {}", libname);
            println!("cargo:rustc-link-lib={}", libname);
        }
    } else {
        // Try all possible library names
        for name in &[
            format!("python{}", version),
            format!("python{}.{}", version, python_info["version_info"]["micro"].as_i64().unwrap()),
            format!("python{}", version.replace(".", "")),
        ] {
            eprintln!("Trying library name: {}", name);
            println!("cargo:rustc-link-lib=shared:{}", name);
        }
    }

    println!("cargo:rerun-if-env-changed=PYTHON_SYS_EXECUTABLE");
    println!("cargo:rerun-if-env-changed=PYTHONHOME");
    println!("cargo:rerun-if-changed=PATH");
    println!("cargo:rerun-if-changed=PYTHONPATH");

    Ok(())
}