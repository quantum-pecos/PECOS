// PECOS/crates/pecos-engines/src/qir/bindings.rs
use lazy_static::lazy_static;
use log::{debug, trace};
use std::collections::VecDeque;
use std::io::{self, Write};
use std::sync::Mutex;

use pecos_core::types::{GateType, QuantumCommand};

lazy_static! {
    // A thread-safe global queue to store quantum commands
    static ref COMMAND_QUEUE: Mutex<VecDeque<QuantumCommand>> = Mutex::new(VecDeque::new());
}

/// Represents a quantum measurement result.
///
/// This struct is an opaque placeholder, as the internal details of a measurement
/// result are not meant to be exposed. Instead, it is used as a reference in
/// quantum runtime functions to store and manage measurement outcomes.
#[repr(C)]
pub struct Result {
    _private: [u8; 0],
}

/// Represents a quantum bit (qubit) in the quantum system.
///
/// This structure is defined as an empty opaque struct to prevent users from
/// directly manipulating qubit internals. Instead, it is intended to be used
/// as a pointer in function calls within the quantum runtime.
#[repr(C)]
pub struct Qubit {
    _private: [u8; 0],
}

/// Represents the RZ rotation gate on the specified qubit and queues it for execution.
///
/// # Arguments
///
/// * `theta` - The rotation angle in radians.
/// * `qubit` - A pointer to the qubit on which the RZ gate will be applied.
///
/// # Panics
///
/// This function will panic if:
/// - The `qubit` pointer is invalid or cannot be converted to a valid index.
/// - The global `COMMAND_QUEUE` mutex is poisoned.
///
/// # Safety
///
/// The `qubit` pointer must be valid and not null. Behavior is undefined if this condition is not met.
#[no_mangle]
pub extern "C" fn __quantum__qis__rz__body(theta: f64, qubit: *const Qubit) {
    let qubit_idx = usize::try_from(qubit as u64).expect("Invalid RZ qubit pointer");

    if let Ok(mut queue) = COMMAND_QUEUE.lock() {
        let cmd = QuantumCommand {
            gate: GateType::RZ { theta },
            qubits: vec![qubit_idx],
        };
        trace!("Queueing RZ gate: {:?}", cmd);
        queue.push_back(cmd);
    }
}

/// Represents the R1XY rotation gate on the specified qubit and queues it for execution.
///
/// # Arguments
///
/// * `phi` - The azimuthal angle in radians.
/// * `theta` - The polar angle in radians.
/// * `qubit` - A pointer to the qubit on which the R1XY gate will be applied.
///
/// # Panics
///
/// This function will panic if:
/// - The `qubit` pointer is invalid or cannot be converted to a valid index.
/// - The global `COMMAND_QUEUE` mutex is poisoned.
///
/// # Safety
///
/// The `qubit` pointer must be valid and not null. Behavior is undefined if this condition is not met.
#[no_mangle]
pub extern "C" fn __quantum__qis__rxy__body(phi: f64, theta: f64, qubit: *const Qubit) {
    let qubit_idx = usize::try_from(qubit as u64).expect("Invalid R1XY qubit pointer");

    if let Ok(mut queue) = COMMAND_QUEUE.lock() {
        let cmd = QuantumCommand {
            gate: GateType::R1XY { phi, theta },
            qubits: vec![qubit_idx],
        };
        trace!("Queueing R1XY gate: {:?}", cmd);
        queue.push_back(cmd);
    }
}

/// Represents the SZZ gate applied to two specified qubits and queues it for execution.
///
/// # Arguments
///
/// * `qubit1` - A pointer to the first qubit.
/// * `qubit2` - A pointer to the second qubit.
///
/// # Panics
///
/// This function will panic if:
/// - The `qubit1` or `qubit2` pointer is invalid or cannot be converted to a valid index.
/// - The global `COMMAND_QUEUE` mutex is poisoned.
///
/// # Safety
///
/// Both `qubit1` and `qubit2` pointers must be valid and not null. Undefined behavior may occur if these conditions are not met.
#[no_mangle]
pub extern "C" fn __quantum__qis__zz__body(qubit1: *const Qubit, qubit2: *const Qubit) {
    let qubit1_idx = usize::try_from(qubit1 as u64).expect("Invalid ZZ qubit1 pointer");
    let qubit2_idx = usize::try_from(qubit2 as u64).expect("Invalid ZZ qubit2 pointer");

    if let Ok(mut queue) = COMMAND_QUEUE.lock() {
        let cmd = QuantumCommand {
            gate: GateType::SZZ,
            qubits: vec![qubit1_idx, qubit2_idx],
        };
        trace!("Queueing SZZ gate: {:?}", cmd);
        queue.push_back(cmd);
    }
}

/// Applies a Hadamard (H) gate to the specified qubit and queues it for execution.
///
/// # Arguments
///
/// * `qubit` - A pointer to the qubit on which the H gate will be applied.
///
/// # Panics
///
/// This function will panic if:
/// - The `qubit` pointer is invalid or cannot be converted to a valid index.
/// - The global `COMMAND_QUEUE` mutex is poisoned.
///
/// # Safety
///
/// The `qubit` pointer must be valid and not null. Behavior is undefined if this condition is not met.
#[no_mangle]
pub extern "C" fn __quantum__qis__h__body(qubit: *const Qubit) {
    let qubit_idx = usize::try_from(qubit as u64).expect("Invalid H qubit pointer");

    if let Ok(mut queue) = COMMAND_QUEUE.lock() {
        let cmd = QuantumCommand {
            gate: GateType::H,
            qubits: vec![qubit_idx],
        };
        trace!("Queueing H gate: {:?}", cmd);
        queue.push_back(cmd);
    }
}

/// Applies a controlled-X (CX) gate to the specified qubits and queues it for execution.
///
/// # Arguments
///
/// * `control` - A pointer to the control qubit.
/// * `target` - A pointer to the target qubit.
///
/// # Panics
///
/// This function will panic if:
/// - The `control` or `target` pointers are invalid or cannot be converted to valid indices.
/// - The global `COMMAND_QUEUE` mutex is poisoned.
///
/// # Safety
///
/// Both `control` and `target` pointers must be valid and not null. Undefined behavior may occur if these conditions are not met.
#[no_mangle]
pub extern "C" fn __quantum__qis__cx__body(control: *const Qubit, target: *const Qubit) {
    let control_idx = usize::try_from(control as u64).expect("Invalid CX control pointer");
    let target_idx = usize::try_from(target as u64).expect("Invalid CX target pointer");

    if let Ok(mut queue) = COMMAND_QUEUE.lock() {
        let cmd = QuantumCommand {
            gate: GateType::CX,
            qubits: vec![control_idx, target_idx],
        };
        trace!("Queueing CX gate: {:?}", cmd);
        queue.push_back(cmd);
    }
}

/// Queues a measurement operation on the specified qubit and associates it with a result.
///
/// # Arguments
///
/// * `qubit` - A pointer to the qubit to be measured. The pointer must be valid and not null.
/// * `result` - A pointer to the Result structure that will store the measurement result. The pointer must be valid and not null.
///
/// # Panics
///
/// This function will panic if:
/// - The `qubit` or `result` pointers are invalid or cannot be converted to valid indices.
/// - The global `COMMAND_QUEUE` mutex is poisoned.
///
/// # Safety
///
/// Both `qubit` and `result` pointers must be valid and not null. Undefined behavior may occur if these conditions are not met.
#[no_mangle]
pub extern "C" fn __quantum__qis__m__body(qubit: *const Qubit, result: *const Result) {
    let qubit_idx = usize::try_from(qubit as u64).expect("Invalid Measurement qubit pointer");
    let result_idx = usize::try_from(result as u64).expect("Invalid Measurement result pointer");

    if let Ok(mut queue) = COMMAND_QUEUE.lock() {
        let cmd = QuantumCommand {
            gate: GateType::Measure {
                result_id: result_idx,
            },
            qubits: vec![qubit_idx],
        };
        trace!("Queueing measurement: {:?}", cmd);
        queue.push_back(cmd);
    }
}

/// Records the result of a quantum measurement and outputs it.
///
/// This function finalizes the current quantum operations by flushing the command queue.
/// It processes any pending commands by printing them to standard output, waits for external input
/// (representing a measurement result), and then associates the provided result pointer
/// with the parsed measurement.
///
/// # Arguments
///
/// * `result` - A pointer to the `Result` structure where the measurement result will be stored.
///              This pointer must be valid and non-null.
/// * `_label` - A pointer to a null-terminated C-style string representing an optional label for
///              the result (currently unused in this implementation).
///
/// # Behavior
///
/// 1. Flushes the `COMMAND_QUEUE` by printing queued commands to the standard output.
///    Commands are formatted using the `format_command` function.
/// 2. Waits for a line of input from the standard input, which is expected to represent
///    a measurement result as an integer.
/// 3. Associates the parsed measurement result with the given `result` pointer and outputs
///    the result.
///
/// # Panics
///
/// This function will panic if:
/// - The `result` pointer is invalid or cannot be converted to a valid index.
/// - The queue mutex (`COMMAND_QUEUE`) is poisoned.
///
/// # Errors
///
/// - If the input from the standard input cannot be parsed as a `u32`, an error will be printed
///   using the following format:
///   `[ERROR] Failed to parse measurement: <error_message>`
///
/// # Safety
///
/// The `result` pointer must be valid and not null. Undefined behavior may occur if this
/// condition is not met.
#[no_mangle]
pub extern "C" fn __quantum__rt__result_record_output(result: *const Result, _label: *const i8) {
    let result_idx = usize::try_from(result as u64).expect("Invalid result pointer");

    if let Ok(mut queue) = COMMAND_QUEUE.lock() {
        if !queue.is_empty() {
            debug!("Flushing {} commands", queue.len());

            println!("FLUSH_BEGIN");
            while let Some(cmd) = queue.pop_front() {
                use crate::channels::stdio::format_command;
                let cmd_str = format_command(&cmd);
                println!("CMD {cmd_str}");
                io::stdout().flush().unwrap();
            }
            println!("FLUSH_END");
            io::stdout().flush().unwrap();
        }

        let mut line = String::new();
        io::stdin().read_line(&mut line).unwrap();

        match line.trim().parse::<u32>() {
            Ok(measurement) => {
                println!("RESULT measurement_{result_idx} {measurement}");
            }
            Err(e) => {
                println!("[ERROR] Failed to parse measurement: {e}");
            }
        }
    }
}
