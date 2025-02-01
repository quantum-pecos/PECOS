%Result = type opaque
%Qubit = type opaque

declare void @__quantum__qis__rz__body(double, %Qubit*)
declare void @__quantum__qis__cx__body(%Qubit*, %Qubit*)
declare void @__quantum__qis__m__body(%Qubit*, %Result*)
declare void @__quantum__rt__result_record_output(%Result*, i8*)

define void @main() #0 {
    ; Apply Hadamard to first qubit using RZ
    call void @__quantum__qis__rz__body(double 3.14159265359, %Qubit* null)
    call void @__quantum__qis__rz__body(double 1.57079632679, %Qubit* null)

    ; Apply CX between qubits
    call void @__quantum__qis__cx__body(%Qubit* null, %Qubit* inttoptr (i64 1 to %Qubit*))

    ; Measure both qubits
    call void @__quantum__qis__m__body(%Qubit* null, %Result* inttoptr (i64 0 to %Result*))
    call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))

    ; Record the results
    call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
    call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)

    ret void
}

attributes #0 = { "EntryPoint" }
