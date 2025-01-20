%Result = type opaque
%Qubit = type opaque

declare void @__quantum__qis__rz__body(double, %Qubit*)
declare void @__quantum__qis__rxy__body(double, double, %Qubit*)
declare void @__quantum__qis__zz__body(%Qubit*, %Qubit*)
declare void @__quantum__qis__m__body(%Qubit*, %Result*)
declare void @__quantum__rt__result_record_output(%Result*, i8*)

define void @main() #0 {
    ; Apply some gates
    call void @__quantum__qis__rz__body(double 3.14, %Qubit* null)
    call void @__quantum__qis__rxy__body(double 3.14, double 1.07, %Qubit* inttoptr (i64 1 to %Qubit*))
    call void @__quantum__qis__zz__body(%Qubit* null, %Qubit* inttoptr (i64 1 to %Qubit*))

    ; Measure both qubits
    call void @__quantum__qis__m__body(%Qubit* null, %Result* inttoptr (i64 0 to %Result*))
    call void @__quantum__qis__m__body(%Qubit* inttoptr (i64 1 to %Qubit*), %Result* inttoptr (i64 1 to %Result*))

    ; Record the results
    call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 0 to %Result*), i8* null)
    call void @__quantum__rt__result_record_output(%Result* inttoptr (i64 1 to %Result*), i8* null)

    ret void
}

attributes #0 = { "EntryPoint" }