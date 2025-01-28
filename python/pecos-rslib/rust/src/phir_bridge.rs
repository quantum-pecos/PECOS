// PECOS/python/pecos-rslib/rust/src/phir_bridge.rs
use parking_lot::Mutex;
use pecos::prelude::*;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;
use std::error::Error;

#[pyclass(module = "_pecos_rslib")]
#[derive(Debug)]
pub struct PHIREngine {
    interpreter: Mutex<PyObject>,
    results: Mutex<HashMap<String, u32>>,
}

#[pymethods]
impl PHIREngine {
    #[new]
    pub fn py_new(phir_json: &str) -> PyResult<Self> {
        Python::with_gil(|py| {
            let pecos = py.import("pecos.classical_interpreters")?;
            let interpreter_cls = pecos.getattr("PHIRClassicalInterpreter")?;
            let interpreter = interpreter_cls.call0()?;
            interpreter.call_method1("init", (phir_json,))?;

            Ok(Self {
                interpreter: Mutex::new(interpreter.into()),
                results: Mutex::new(HashMap::new()),
            })
        })
    }

    #[getter]
    fn results_dict(&self, py: Python<'_>) -> Py<PyAny> {
        let results = self.results.lock();
        PyObject::from(
            results
                .clone()
                .into_pyobject(py)
                .expect("Failed to convert results"),
        )
    }

    fn process_program(&mut self) -> PyResult<Vec<PyObject>> {
        Python::with_gil(|py| {
            ClassicalEngine::process_program(self)
                .map_err(|e| QueueErrorWrapper(e).into())
                .and_then(|commands| {
                    let mut py_commands = Vec::with_capacity(commands.len());
                    for cmd in commands {
                        let py_dict = PyDict::new(py);

                        // Create a dict for parameters
                        let params_dict = PyDict::new(py);

                        // Convert gate type and parameters
                        match cmd.gate {
                            GateType::Measure { result_id } => {
                                py_dict.set_item("gate_type", "Measure")?;
                                params_dict.set_item("result_id", result_id)?;
                            }
                            GateType::RZ { theta } => {
                                py_dict.set_item("gate_type", "RZ")?;
                                params_dict.set_item("theta", theta)?;
                            }
                            GateType::R1XY { phi, theta } => {
                                py_dict.set_item("gate_type", "R1XY")?;
                                let angles = vec![phi, theta];
                                params_dict.set_item("angles", angles)?;
                            }
                            GateType::SZZ => {
                                py_dict.set_item("gate_type", "SZZ")?;
                            }
                            GateType::H => {
                                py_dict.set_item("gate_type", "H")?;
                            }
                            GateType::CX => {
                                py_dict.set_item("gate_type", "CX")?;
                            }
                        }

                        py_dict.set_item("params", params_dict)?;
                        py_dict.set_item("qubits", cmd.qubits)?;

                        // Convert to PyObject
                        let py_obj: PyObject = py_dict.into_any().into();
                        py_commands.push(py_obj);
                    }
                    Ok(py_commands)
                })
        })
    }

    fn handle_measurement(&mut self, measurement: u32) -> PyResult<()> {
        Python::with_gil(|py| {
            let interpreter = self.interpreter.lock();
            let dict = PyDict::new(py);
            dict.set_item("measurement", measurement)?;
            let results_guard = self.results.lock();
            let dict_list: Vec<PyObject> = results_guard
                .iter()
                .map(|(key, value)| {
                    let py_dict = PyDict::new(py);
                    py_dict.set_item("key", key).expect("Failed to set key");
                    py_dict
                        .set_item("value", value)
                        .expect("Failed to set value");
                    py_dict.into_any().into()
                })
                .collect();

            interpreter.call_method1(py, "receive_results", (dict_list,))?;
            Ok(())
        })
    }

    fn get_results(&self) -> PyResult<HashMap<String, u32>> {
        match ClassicalEngine::get_results(self) {
            Ok(results) => Ok(results.measurements),
            Err(e) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                e.to_string(),
            )),
        }
    }
}

impl ClassicalEngine for PHIREngine {
    fn process_program(&mut self) -> Result<CommandBatch, QueueError> {
        Python::with_gil(|py| {
            let interpreter = self.interpreter.lock();
            let program = interpreter.getattr(py, "program")?;
            let ops = program.getattr(py, "ops")?;
            let result = interpreter.call_method1(py, "execute", (ops,))?;

            match result.call_method0(py, "__next__") {
                Ok(commands) if commands.is_none(py) => Ok(vec![]),
                Ok(commands) => {
                    let py_list = commands.downcast_bound::<PyList>(py)?;
                    let mut batch = Vec::new();
                    for py_cmd in py_list.iter() {
                        let (gate, qubits) = convert_gate(&py_cmd)?;
                        batch.push(QuantumCommand { gate, qubits });
                    }
                    Ok(batch)
                }
                Err(_) => Ok(vec![]),
            }
        })
        .map_err(|e: PyErr| QueueError::ExecutionError(e.to_string()))
    }

    fn handle_measurement(&mut self, measurement: MeasurementResult) -> Result<(), QueueError> {
        Python::with_gil(|py| {
            let interpreter = self.interpreter.lock();
            let dict = PyDict::new(py);
            dict.set_item("measurement", measurement)?;
            let results_guard = self.results.lock();
            let dict_list: Vec<_> = results_guard
                .iter()
                .map(|(key, value)| {
                    let py_dict = PyDict::new(py);
                    py_dict.set_item("key", key).expect("Failed to set key");
                    py_dict
                        .set_item("value", value)
                        .expect("Failed to set value");
                    py_dict
                        .into_pyobject(py)
                        .expect("Failed to convert py_dict")
                })
                .collect();

            interpreter.call_method1(py, "receive_results", (dict_list,))?;

            Ok(())
        })
        .map_err(|e: PyErr| QueueError::ExecutionError(e.to_string()))
    }

    fn get_results(&self) -> Result<ShotResult, QueueError> {
        Python::with_gil(|py| {
            let interpreter = self.interpreter.lock();
            let py_results = interpreter.call_method0(py, "results")?;
            let results: HashMap<String, u32> = py_results.extract(py)?;
            *self.results.lock() = results.clone();
            Ok(ShotResult {
                measurements: results,
            })
        })
        .map_err(|e: PyErr| QueueError::ExecutionError(e.to_string()))
    }

    fn compile(&self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}

fn convert_gate(py_cmd: &Bound<'_, PyAny>) -> Result<(GateType, Vec<usize>), PyErr> {
    let name: String = py_cmd.getattr("name")?.extract()?;
    let args = py_cmd.getattr("args")?;

    let mut qubits = Vec::new();
    for item in args.try_iter()? {
        let item = item?;
        let qubit_idx = if item.is_instance_of::<PyList>() {
            item.get_item(1)?.extract()?
        } else {
            item.extract()?
        };
        qubits.push(qubit_idx);
    }

    let gate = match name.as_str() {
        "RZ" => {
            let angles: Vec<f64> = py_cmd.getattr("angles")?.extract()?;
            GateType::RZ { theta: angles[0] }
        }
        "R1XY" => {
            let angles: Vec<f64> = py_cmd.getattr("angles")?.extract()?;
            GateType::R1XY {
                phi: angles[0],
                theta: angles[1],
            }
        }
        "SZZ" => GateType::SZZ,
        "H" => GateType::H,
        "CX" => GateType::CX,
        "Measure" => {
            let returns = py_cmd.getattr("returns")?;
            let return_item = returns.get_item(0)?;
            let result_id: usize = return_item.get_item(1)?.extract()?;
            GateType::Measure { result_id }
        }
        _ => {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Unsupported gate type: {name}"
            )))
        }
    };

    Ok((gate, qubits))
}

// Newtype wrapper for QueueError
#[derive(Debug)]
struct QueueErrorWrapper(QueueError);

// Implement conversion from QueueErrorWrapper to PyErr
impl From<QueueErrorWrapper> for PyErr {
    fn from(err: QueueErrorWrapper) -> PyErr {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(err.0.to_string())
    }
}
