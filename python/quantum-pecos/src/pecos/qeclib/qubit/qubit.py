from pecos.slr import Qubit, Bit
from pecos.qeclib.qubit import sq_paulis, sq_sqrt_paulis, sq_hadamards, tq_cliffords, measures, preps

# TODO accept multiple arguments like the underlying implementations

class PhysicalQubit:

    @staticmethod
    def x(*qargs):
        """Pauli X gate"""
        return sq_paulis.X(*qargs)

    @staticmethod
    def y(*qargs):
        """Pauli Y gate"""
        return sq_paulis.Y(*qargs)

    @staticmethod
    def z(*qargs):
        """Pauli Z gate"""
        return sq_paulis.Z(*qargs)

    @staticmethod
    def sz(*qargs):
        """Sqrt of Pauli Z gate"""
        return sq_sqrt_paulis.SZ(*qargs)

    @staticmethod
    def h(*qargs):
        """Hadamard gate"""
        return sq_hadamards.H(*qargs)

    @staticmethod
    def cx(*qargs):
        """Controlled-X gate"""
        return tq_cliffords.CX(*qargs)

    @staticmethod
    def cy(*qargs):
        """Controlled-X gate"""
        return tq_cliffords.CX(*qargs)

    @staticmethod
    def cz(*qargs):
        """Controlled-X gate"""
        return tq_cliffords.CX(*qargs)

    @staticmethod
    def pz(*qargs):
        """Measurement gate"""
        return preps.Prep(*qargs)

    @staticmethod
    def mz(qubits, outputs):
        """Measurement gate"""
        return measures.Measure(*qubits) > outputs
