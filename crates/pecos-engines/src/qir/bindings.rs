// PECOS/crates/pecos-engines/src/qir/bindings.rs
use lazy_static::lazy_static;
use log::{debug, trace};
use std::collections::VecDeque;
use std::io::{self, Write};
use std::sync::Mutex;

use crate::types::{GateType, QuantumCommand};

lazy_static! {
    static ref COMMAND_QUEUE: Mutex<VecDeque<QuantumCommand>> = Mutex::new(VecDeque::new());
}

#[repr(C)]
pub struct Result {
    _private: [u8; 0],
}

#[repr(C)]
pub struct Qubit {
    _private: [u8; 0],
}

#[no_mangle]
pub extern "C" fn __quantum__qis__rz__body(theta: f64, qubit: *const Qubit) {
    let qubit_idx = (qubit as u64) as usize;

    if let Ok(mut queue) = COMMAND_QUEUE.lock() {
        let cmd = QuantumCommand {
            gate: GateType::RZ { theta },
            qubits: vec![qubit_idx],
        };
        trace!("Queueing RZ gate: {:?}", cmd);
        queue.push_back(cmd);
    }
}

#[no_mangle]
pub extern "C" fn __quantum__qis__rxy__body(phi: f64, theta: f64, qubit: *const Qubit) {
    let qubit_idx = (qubit as u64) as usize;

    if let Ok(mut queue) = COMMAND_QUEUE.lock() {
        let cmd = QuantumCommand {
            gate: GateType::R1XY { phi, theta },
            qubits: vec![qubit_idx],
        };
        trace!("Queueing R1XY gate: {:?}", cmd);
        queue.push_back(cmd);
    }
}

#[no_mangle]
pub extern "C" fn __quantum__qis__zz__body(qubit1: *const Qubit, qubit2: *const Qubit) {
    let qubit1_idx = (qubit1 as u64) as usize;
    let qubit2_idx = (qubit2 as u64) as usize;

    if let Ok(mut queue) = COMMAND_QUEUE.lock() {
        let cmd = QuantumCommand {
            gate: GateType::SZZ,
            qubits: vec![qubit1_idx, qubit2_idx],
        };
        trace!("Queueing SZZ gate: {:?}", cmd);
        queue.push_back(cmd);
    }
}

#[no_mangle]
pub extern "C" fn __quantum__qis__h__body(qubit: *const Qubit) {
    let qubit_idx = (qubit as u64) as usize;

    if let Ok(mut queue) = COMMAND_QUEUE.lock() {
        let cmd = QuantumCommand {
            gate: GateType::H,
            qubits: vec![qubit_idx],
        };
        trace!("Queueing H gate: {:?}", cmd);
        queue.push_back(cmd);
    }
}

#[no_mangle]
pub extern "C" fn __quantum__qis__cx__body(control: *const Qubit, target: *const Qubit) {
    let control_idx = (control as u64) as usize;
    let target_idx = (target as u64) as usize;

    if let Ok(mut queue) = COMMAND_QUEUE.lock() {
        let cmd = QuantumCommand {
            gate: GateType::CX,
            qubits: vec![control_idx, target_idx],
        };
        trace!("Queueing CX gate: {:?}", cmd);
        queue.push_back(cmd);
    }
}

#[no_mangle]
pub extern "C" fn __quantum__qis__m__body(qubit: *const Qubit, result: *const Result) {
    let qubit_idx = (qubit as u64) as usize;
    let result_idx = (result as u64) as usize;

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

#[no_mangle]
pub extern "C" fn __quantum__rt__result_record_output(result: *const Result, _label: *const i8) {
    let result_idx = (result as u64) as usize;

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
