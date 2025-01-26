#!/bin/bash

# Get Python paths
UV_PYTHON=$(uv run which python)
PYTHON_ROOT="/home/ciaranra/.local/share/uv/python/cpython-3.13.1-linux-x86_64-gnu"
PYTHON_LIB_PATH="$PYTHON_ROOT/lib"
VENV_PATH="/home/ciaranra/Repos/PECOS/.venv"

# Export Python environment variables
export UV_PYTHON
export PYTHON_SYS_EXECUTABLE=$UV_PYTHON

# Don't set PYTHONHOME, instead manage PYTHONPATH explicitly
unset PYTHONHOME

# Build up PYTHONPATH with all necessary components
PYTHONPATH_COMPONENTS=(
    "$PYTHON_ROOT/lib/python3.13"
    "$PYTHON_ROOT/lib/python3.13/lib-dynload"
    "$VENV_PATH/lib/python3.13/site-packages"
    "/home/ciaranra/Repos/PECOS/python/quantum-pecos/src"
    "/home/ciaranra/Repos/PECOS/python/pecos-rslib/src"
)

export PYTHONPATH=$(IFS=:; echo "${PYTHONPATH_COMPONENTS[*]}")

# Add Python lib directory to library path
export LD_LIBRARY_PATH="$PYTHON_ROOT/lib:$LD_LIBRARY_PATH"

# Print diagnostic information
echo "Python Environment:"
echo "==================="
echo "PYTHONPATH: $PYTHONPATH"
echo "LD_LIBRARY_PATH: $LD_LIBRARY_PATH"
echo "PYTHON_SYS_EXECUTABLE: $PYTHON_SYS_EXECUTABLE"
echo "UV_PYTHON: $UV_PYTHON"
echo "PYTHON_SYS_EXECUTABLE: $PYTHON_SYS_EXECUTABLE"
echo "LD_LIBRARY_PATH: $LD_LIBRARY_PATH"
echo -e "\nPYTHONPATH components:"
for component in "${PYTHONPATH_COMPONENTS[@]}"; do
    echo "  $component"
    if [ -d "$component" ]; then
        echo "    ✓ Directory exists"
    else
        echo "    ✗ Directory not found"
    fi
done

# Run a quick Python test
echo -e "\nPython import test:"
$UV_PYTHON -c "
import os
import sys
import pecos
print(f'Python executable: {sys.executable}')
print(f'PECOS location: {pecos.__file__}')
print('Python path:')
for p in sys.path:
    print(f'  {p}')
"

# Run the requested command
echo -e "\nRunning command:"
echo "==================="
exec "$@"