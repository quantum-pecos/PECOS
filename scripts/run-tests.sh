#!/bin/bash
set -e  # Exit on error
set -x  # Print commands as they execute

# Find PECOS root
if [[ -d .git && -f pyproject.toml ]]; then
    PECOS_ROOT="$(pwd)"
elif [[ -d ../../../.git && -f ../../../pyproject.toml ]]; then
    PECOS_ROOT="$(cd ../../.. && pwd)"
else
    echo "Must be run from PECOS root or a subdirectory"
    exit 1
fi

echo "Found PECOS root at: $PECOS_ROOT"

# Set up Python environment
UV_PYTHON="$PECOS_ROOT/.venv/bin/python"
export PATH="$PECOS_ROOT/.venv/bin:$PATH"
export PYTHON_SYS_EXECUTABLE="$UV_PYTHON"

# First install pecos
cd "$PECOS_ROOT/python/quantum-pecos"
uv pip install -e .[all]

# Then build and install rslib with maturin
cd "$PECOS_ROOT/python/pecos-rslib"
maturin develop --uv

# Get Python paths and info
echo "=== Python Environment ==="
UV_LIB_PATH=$("$UV_PYTHON" -c 'import sys, os; print(os.path.join(os.path.expanduser("~"), ".local/share/uv/python", f"cpython-{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}-linux-x86_64-gnu/lib"))')
export LD_LIBRARY_PATH="$UV_LIB_PATH:$LD_LIBRARY_PATH"

echo "Python Library Path: $UV_LIB_PATH"
ls -l "$UV_LIB_PATH"/libpython*
ldd "$UV_LIB_PATH"/libpython*

# Export verbose flags for Rust
export RUSTFLAGS="-C link-arg=-Wl,--verbose -C link-arg=-Wl,-rpath,$UV_LIB_PATH -L$UV_LIB_PATH -lpython3.13"

# Run tests from rust directory
cd rust
cargo test --features python-tests -vv -- --nocapture
