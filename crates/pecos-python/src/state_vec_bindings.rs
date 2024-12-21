use pecos::prelude::*;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

/// The struct represents the state-vector simulator exposed to Python
#[pyclass]
pub struct RsStateVec {
    inner: StateVec,
}

#[pymethods]
impl RsStateVec {
    /// Creates a new state-vector simulator with the specified number of qubits
    #[new]
    pub fn new(num_qubits: usize) -> Self {
        RsStateVec {
            inner: StateVec::new(num_qubits),
        }
    }

    /// Resets the quantum state to the all-zero state
    fn reset(&mut self) {
        self.inner.reset();
    }

    /// Executes a single-qubit gate based on the provided symbol and location
    ///
    /// `symbol`: The gate symbol (e.g., "X", "H", "Z")
    /// `location`: The qubit index to apply the gate to
    /// `params`: Optional parameters for parameterized gates (currently unused here)
    ///
    /// Returns an optional result, usually `None` unless a measurement is performed
    #[allow(clippy::too_many_lines)]
    #[pyo3(signature = (symbol, location, _params=None))]
    fn run_1q_gate(
        &mut self,
        symbol: &str,
        location: usize,
        _params: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Option<u8>> {
        match symbol {
            "X" => {
                self.inner.x(location);
                Ok(None)
            }
            "Y" => {
                self.inner.y(location);
                Ok(None)
            }
            "Z" => {
                self.inner.z(location);
                Ok(None)
            }
            "H" => {
                self.inner.h(location);
                Ok(None)
            }
            "H2" => {
                self.inner.h2(location);
                Ok(None)
            }
            "H3" => {
                self.inner.h3(location);
                Ok(None)
            }
            "H4" => {
                self.inner.h4(location);
                Ok(None)
            }
            "H5" => {
                self.inner.h5(location);
                Ok(None)
            }
            "H6" => {
                self.inner.h6(location);
                Ok(None)
            }
            "F" => {
                self.inner.f(location);
                Ok(None)
            }
            "Fdg" => {
                self.inner.fdg(location);
                Ok(None)
            }
            "F2" => {
                self.inner.f2(location);
                Ok(None)
            }
            "F2dg" => {
                self.inner.f2dg(location);
                Ok(None)
            }
            "F3" => {
                self.inner.f3(location);
                Ok(None)
            }
            "F3dg" => {
                self.inner.f3dg(location);
                Ok(None)
            }
            "F4" => {
                self.inner.f4(location);
                Ok(None)
            }
            "F4dg" => {
                self.inner.f4dg(location);
                Ok(None)
            }
            "SX" => {
                self.inner.sx(location);
                Ok(None)
            }
            "SXdg" => {
                self.inner.sxdg(location);
                Ok(None)
            }
            "SY" => {
                self.inner.sy(location);
                Ok(None)
            }
            "SYdg" => {
                self.inner.sydg(location);
                Ok(None)
            }
            "SZ" => {
                self.inner.sz(location);
                Ok(None)
            }
            "SZdg" => {
                self.inner.szdg(location);
                Ok(None)
            }
            "PZ" => {
                self.inner.pz(location);
                Ok(None)
            }
            "PX" => {
                self.inner.px(location);
                Ok(None)
            }
            "PY" => {
                self.inner.py(location);
                Ok(None)
            }
            "PnZ" => {
                self.inner.pnz(location);
                Ok(None)
            }
            "PnX" => {
                self.inner.pnx(location);
                Ok(None)
            }
            "PnY" => {
                self.inner.pny(location);
                Ok(None)
            }
            "MZ" | "MX" | "MY" => {
                let result = match symbol {
                    "MZ" => self.inner.mz(location),
                    "MX" => self.inner.mx(location),
                    "MY" => self.inner.my(location),
                    _ => unreachable!(),
                };
                Ok(Some(u8::from(result.outcome)))
            }
            _ => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Unsupported single-qubit gate",
            )),
        }
    }

    /// Executes a two-qubit gate based on the provided symbol and locations
    ///
    /// `symbol`: The gate symbol (e.g., "CX", "CZ")
    /// `location`: A tuple specifying the two qubits to apply the gate to
    /// `params`: Optional parameters for parameterized gates (currently unused here)
    ///
    /// Returns an optional result, usually `None` unless a measurement is performed
    #[pyo3(signature = (symbol, location, _params))]
    fn run_2q_gate(
        &mut self,
        symbol: &str,
        location: &Bound<'_, PyTuple>,
        _params: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Option<u8>> {
        if location.len() != 2 {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Two-qubit gate requires exactly 2 qubit locations",
            ));
        }

        let q1: usize = location.get_item(0)?.extract()?;
        let q2: usize = location.get_item(1)?.extract()?;

        match symbol {
            "CX" => {
                self.inner.cx(q1, q2);
                Ok(None)
            }
            "CY" => {
                self.inner.cy(q1, q2);
                Ok(None)
            }
            "CZ" => {
                self.inner.cz(q1, q2);
                Ok(None)
            }
            "SXX" => {
                self.inner.sxx(q1, q2);
                Ok(None)
            }
            "SXXdg" => {
                self.inner.sxxdg(q1, q2);
                Ok(None)
            }
            "SYY" => {
                self.inner.syy(q1, q2);
                Ok(None)
            }
            "SYYdg" => {
                self.inner.syydg(q1, q2);
                Ok(None)
            }
            "SZZ" => {
                self.inner.szz(q1, q2);
                Ok(None)
            }
            "SZZdg" => {
                self.inner.szzdg(q1, q2);
                Ok(None)
            }
            "SWAP" => {
                self.inner.swap(q1, q2);
                Ok(None)
            }
            "G2" => {
                self.inner.g2(q1, q2);
                Ok(None)
            }
            _ => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Unsupported two-qubit gate",
            )),
        }
    }

    /// Dispatches a gate to the appropriate handler based on the number of qubits specified
    ///
    /// `symbol`: The gate symbol
    /// `location`: A tuple specifying the qubits to apply the gate to
    /// `params`: Optional parameters for parameterized gates
    #[pyo3(signature = (symbol, location, params=None))]
    fn run_gate(
        &mut self,
        symbol: &str,
        location: &Bound<'_, PyTuple>,
        params: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Option<u8>> {
        match location.len() {
            1 => {
                let qubit: usize = location.get_item(0)?.extract()?;
                self.run_1q_gate(symbol, qubit, params)
            }
            2 => self.run_2q_gate(symbol, location, params),
            _ => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Gate location must be specified for either 1 or 2 qubits",
            )),
        }
    }
}
