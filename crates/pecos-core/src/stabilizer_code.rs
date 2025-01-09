use crate::qubit_id::QubitId;

pub trait StabilizerCode {
    fn num_data_qubits(&self) -> usize;

    // Get a vector of the stabilizer generators used for the code.
    // Todo: Utilize PauliSet when available
    fn get_stabilizer_gens(&self) -> Vec<Vec<(usize, QubitId)>>;

    // Get the boundaries of the code
    // TODO: Consider adding identifiers to identify the boundary types
    fn get_boundaries(&self) -> Vec<Vec<QubitId>>;
}
