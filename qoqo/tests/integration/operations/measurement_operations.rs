// Copyright © 2021 HQS Quantum Simulations GmbH. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except
// in compliance with the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the
// License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either
// express or implied. See the License for the specific language governing permissions and
// limitations under the License.

use pyo3::prelude::*;
use qoqo::operations::*;
use qoqo::CircuitWrapper;
use roqoqo::operations::*;
use roqoqo::Circuit;
use std::collections::{HashMap, HashSet};
use test_case::test_case;

fn create_qubit_mapping() -> HashMap<usize, usize> {
    let mut qubit_mapping: HashMap<usize, usize> = HashMap::new();
    qubit_mapping.insert(0, 1);
    qubit_mapping
}

fn qubit_remapping() -> HashMap<usize, usize> {
    let mut qubit_mapping: HashMap<usize, usize> = HashMap::new();
    qubit_mapping.insert(0, 2);
    qubit_mapping
}

fn qubits_remapped() -> HashMap<usize, usize> {
    let mut qubit_mapping: HashMap<usize, usize> = HashMap::new();
    qubit_mapping.insert(2, 1);
    qubit_mapping
}

fn create_circuit() -> Circuit {
    let mut circuit = Circuit::new();
    circuit.add_operation(PauliX::new(0));
    circuit
}

fn circuit_remapped() -> Circuit {
    let mut circuit = Circuit::new();
    circuit.add_operation(PauliX::new(2));
    circuit
}

fn new_circuit(py: Python) -> &PyCell<CircuitWrapper> {
    let circuit_type = py.get_type::<CircuitWrapper>();
    circuit_type
        .call0()
        .unwrap()
        .cast_as::<PyCell<CircuitWrapper>>()
        .unwrap()
}

/// Test readout() input/function
#[test_case(Operation::from(MeasureQubit::new(0, String::from("ro"), 1)); "MeasureQubit")]
#[test_case(Operation::from(PragmaGetStateVector::new(String::from("ro"), Some(create_circuit()))); "PragmaGetStateVector")]
#[test_case(Operation::from(PragmaGetDensityMatrix::new(String::from("ro"), Some(create_circuit()))); "PragmaGetDensityMatrix")]
#[test_case(Operation::from(PragmaGetOccupationProbability::new(String::from("ro"), Some(create_circuit()))); "PragmaGetOccupationProbability")]
#[test_case(Operation::from(PragmaGetPauliProduct::new(create_qubit_mapping(), String::from("ro"), create_circuit())); "PragmaGetPauliProduct")]
#[test_case(Operation::from(PragmaRepeatedMeasurement::new(String::from("ro"), 2, Some(create_qubit_mapping()))); "PragmaRepeatedMeasurement")]
fn test_pyo3_readout(input_measurement: Operation) {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let operation = convert_operation_to_pyobject(input_measurement).unwrap();
        let readout_op: &String =
            &String::extract(operation.call_method0(py, "readout").unwrap().as_ref(py)).unwrap();
        let readout_param: String = String::from("ro");
        assert_eq!(readout_op, &readout_param);
    })
}

/// Test qubit_mapping() or qubit_paulis input/function
#[test_case(Operation::from(PragmaGetPauliProduct::new(create_qubit_mapping(), String::from("ro"), create_circuit())), "qubit_paulis"; "PragmaGetPauliProduct")]
#[test_case(Operation::from(PragmaRepeatedMeasurement::new(String::from("ro"), 2, Some(create_qubit_mapping()))), "qubit_mapping"; "PragmaRepeatedMeasurement")]
fn test_pyo3_qubit_mapping(input_measurement: Operation, operation_name: &str) {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let operation = convert_operation_to_pyobject(input_measurement).unwrap();
        let readout_op: &HashMap<usize, usize> = &HashMap::extract(
            operation
                .call_method0(py, operation_name)
                .unwrap()
                .as_ref(py),
        )
        .unwrap();
        assert_eq!(readout_op, &create_qubit_mapping());
    })
}

/// Test circuit() input/function
#[test_case(Operation::from(PragmaGetStateVector::new(String::from("ro"), Some(create_circuit()))); "PragmaGetStateVector")]
#[test_case(Operation::from(PragmaGetDensityMatrix::new(String::from("ro"), Some(create_circuit()))); "PragmaGetDensityMatrix")]
#[test_case(Operation::from(PragmaGetOccupationProbability::new(String::from("ro"), Some(create_circuit()))); "PragmaGetOccupationProbability")]
#[test_case(Operation::from(PragmaGetPauliProduct::new(create_qubit_mapping(), String::from("ro"), create_circuit())); "PragmaGetPauliProduct")]
fn test_pyo3_circuit(input_measurement: Operation) {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let operation = convert_operation_to_pyobject(input_measurement).unwrap();
        let to_circuit = operation.call_method0(py, "circuit").unwrap();
        let circuit_op = to_circuit.as_ref(py);

        let circuit = new_circuit(py);
        let paulix = convert_operation_to_pyobject(Operation::from(PauliX::new(0))).unwrap();
        circuit.call_method1("add", (paulix,)).unwrap();

        let comparison_circuit =
            bool::extract(circuit_op.call_method1("__eq__", (circuit,)).unwrap()).unwrap();
        assert!(comparison_circuit);
    })
}

/// Test MeasureQubit remaining inputs (qubit, readout_index)
#[test]
fn test_pyo3_input_measurequbit_input() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let operation = convert_operation_to_pyobject(Operation::from(MeasureQubit::new(
            0,
            String::from("ro"),
            1,
        )))
        .unwrap();

        let qubit_op: &usize =
            &usize::extract(operation.call_method0(py, "qubit").unwrap().as_ref(py)).unwrap();
        let qubit_param: &usize = &0;
        assert_eq!(qubit_op, qubit_param);

        let ro_index_op: &usize = &usize::extract(
            operation
                .call_method0(py, "readout_index")
                .unwrap()
                .as_ref(py),
        )
        .unwrap();
        let ro_index_param: &usize = &1;
        assert_eq!(ro_index_op, ro_index_param);
    })
}

/// Test PragmaRepeatedMeasurement remaining input (number_mreasurements)
#[test]
fn test_pyo3_input_pragmarepeatedmeasurements_input() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let operation = convert_operation_to_pyobject(Operation::from(
            PragmaRepeatedMeasurement::new(String::from("ro"), 2, Some(create_qubit_mapping())),
        ))
        .unwrap();

        let nbr_meas_op: &usize = &usize::extract(
            operation
                .call_method0(py, "number_measurements")
                .unwrap()
                .as_ref(py),
        )
        .unwrap();
        let nbr_meas_param: &usize = &2;
        assert_eq!(nbr_meas_op, nbr_meas_param);
    })
}

/// Test involved_qubits function for Pragmas with All
#[test_case(Operation::from(PragmaGetStateVector::new(String::from("ro"), Some(create_circuit()))); "PragmaGetStateVector")]
#[test_case(Operation::from(PragmaGetDensityMatrix::new(String::from("ro"), Some(create_circuit()))); "PragmaGetDensityMatrix")]
#[test_case(Operation::from(PragmaGetOccupationProbability::new(String::from("ro"), Some(create_circuit()))); "PragmaGetOccupationProbability")]
#[test_case(Operation::from(PragmaRepeatedMeasurement::new(String::from("ro"), 2, Some(create_qubit_mapping()))); "PragmaRepeatedMeasurement")]
fn test_pyo3_involved_qubits_all(input_definition: Operation) {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let operation = convert_operation_to_pyobject(input_definition).unwrap();
        let to_involved = operation.call_method0(py, "involved_qubits").unwrap();
        let involved_op: HashSet<&str> = HashSet::extract(to_involved.as_ref(py)).unwrap();
        let mut involved_param: HashSet<&str> = HashSet::new();
        involved_param.insert("All");
        assert_eq!(involved_op, involved_param);
    })
}

/// Test involved_qubits function for pragmas with qubit 0
#[test_case(Operation::from(MeasureQubit::new(0, String::from("ro"), 1)); "MeasureQubit")]
#[test_case(Operation::from(PragmaGetPauliProduct::new(create_qubit_mapping(), String::from("ro"), create_circuit())); "PragmaGetPauliProduct")]
fn test_pyo3_involved_qubits_0(input_definition: Operation) {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let operation = convert_operation_to_pyobject(input_definition).unwrap();
        let to_involved = operation.call_method0(py, "involved_qubits").unwrap();
        let involved_op: HashSet<usize> = HashSet::extract(to_involved.as_ref(py)).unwrap();
        let mut involved_param: HashSet<usize> = HashSet::new();
        involved_param.insert(0);
        assert_eq!(involved_op, involved_param);
    })
}

/// Test format and repr functions
#[test_case(Operation::from(MeasureQubit::new(0, String::from("ro"), 1)), "MeasureQubit { qubit: 0, readout: \"ro\", readout_index: 1 }"; "MeasureQubit")]
#[test_case(Operation::from(PragmaGetStateVector::new(String::from("ro"), None)), "PragmaGetStateVector { readout: \"ro\", circuit: None }"; "PragmaGetStateVector")]
#[test_case(Operation::from(PragmaGetDensityMatrix::new(String::from("ro"), None)), "PragmaGetDensityMatrix { readout: \"ro\", circuit: None }"; "PragmaGetDensityMatrix")]
#[test_case(Operation::from(PragmaGetOccupationProbability::new(String::from("ro"), None)), "PragmaGetOccupationProbability { readout: \"ro\", circuit: None }"; "PragmaGetOccupationProbability")]
#[test_case(Operation::from(PragmaGetPauliProduct::new(create_qubit_mapping(), String::from("ro"), Circuit::default())), "PragmaGetPauliProduct { qubit_paulis: {0: 1}, readout: \"ro\", circuit: Circuit { definitions: [], operations: [], _roqoqo_version: RoqoqoVersion } }"; "PragmaGetPauliProduct")]
#[test_case(Operation::from(PragmaRepeatedMeasurement::new(String::from("ro"), 2, Some(create_qubit_mapping()))), "PragmaRepeatedMeasurement { readout: \"ro\", number_measurements: 2, qubit_mapping: Some({0: 1}) }"; "PragmaRepeatedMeasurement")]
fn test_pyo3_format_repr(input_measurement: Operation, format_repr: &str) {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let operation = convert_operation_to_pyobject(input_measurement).unwrap();
        let to_format = operation.call_method1(py, "__format__", ("",)).unwrap();
        let format_op: &str = <&str>::extract(to_format.as_ref(py)).unwrap();
        let to_repr = operation.call_method0(py, "__repr__").unwrap();
        let repr_op: &str = <&str>::extract(to_repr.as_ref(py)).unwrap();
        assert_eq!(format_op, format_repr);
        assert_eq!(repr_op, format_repr);
    })
}

/// Test copy and deepcopy functions
#[test_case(Operation::from(MeasureQubit::new(0, String::from("ro"), 1)); "MeasureQubit")]
#[test_case(Operation::from(PragmaGetStateVector::new(String::from("ro"), Some(create_circuit()))); "PragmaGetStateVector")]
#[test_case(Operation::from(PragmaGetDensityMatrix::new(String::from("ro"), Some(create_circuit()))); "PragmaGetDensityMatrix")]
#[test_case(Operation::from(PragmaGetOccupationProbability::new(String::from("ro"), Some(create_circuit()))); "PragmaGetOccupationProbability")]
#[test_case(Operation::from(PragmaGetPauliProduct::new(create_qubit_mapping(), String::from("ro"), create_circuit())); "PragmaGetPauliProduct")]
#[test_case(Operation::from(PragmaRepeatedMeasurement::new(String::from("ro"), 2, Some(create_qubit_mapping()))); "PragmaRepeatedMeasurement")]
fn test_pyo3_copy_deepcopy(input_measurement: Operation) {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let operation = convert_operation_to_pyobject(input_measurement).unwrap();
        let copy_op = operation.call_method0(py, "__copy__").unwrap();
        let deepcopy_op = operation.call_method1(py, "__deepcopy__", ("",)).unwrap();
        let copy_deepcopy_param = operation;

        let comparison_copy = bool::extract(
            copy_op
                .as_ref(py)
                .call_method1("__eq__", (copy_deepcopy_param.clone(),))
                .unwrap(),
        )
        .unwrap();
        assert!(comparison_copy);
        let comparison_deepcopy = bool::extract(
            deepcopy_op
                .as_ref(py)
                .call_method1("__eq__", (copy_deepcopy_param,))
                .unwrap(),
        )
        .unwrap();
        assert!(comparison_deepcopy);
    })
}

/// Test tags function
#[test_case(Operation::from(PragmaGetStateVector::new(String::from("ro"), Some(create_circuit()))), "PragmaGetStateVector"; "PragmaGetStateVector")]
#[test_case(Operation::from(PragmaGetDensityMatrix::new(String::from("ro"), Some(create_circuit()))), "PragmaGetDensityMatrix"; "PragmaGetDensityMatrix")]
#[test_case(Operation::from(PragmaGetOccupationProbability::new(String::from("ro"), Some(create_circuit()))), "PragmaGetOccupationProbability"; "PragmaGetOccupationProbability")]
#[test_case(Operation::from(PragmaGetPauliProduct::new(create_qubit_mapping(), String::from("ro"), create_circuit())), "PragmaGetPauliProduct"; "PragmaGetPauliProduct")]
#[test_case(Operation::from(PragmaRepeatedMeasurement::new(String::from("ro"), 2, Some(create_qubit_mapping()))), "PragmaRepeatedMeasurement"; "PragmaRepeatedMeasurement")]
fn test_pyo3_tags(input_measurement: Operation, tag_name: &str) {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let operation = convert_operation_to_pyobject(input_measurement).unwrap();
        let to_tag = operation.call_method0(py, "tags").unwrap();
        let tags_op: &Vec<&str> = &Vec::extract(to_tag.as_ref(py)).unwrap();
        let tags_param: &[&str] = &["Operation", "Measurement", "PragmaOperation", tag_name];
        assert_eq!(tags_op, tags_param);
    })
}

/// Test tags function for MeasureQubit
#[test]
fn test_pyo3_tags() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let operation = convert_operation_to_pyobject(Operation::from(MeasureQubit::new(
            0,
            String::from("ro"),
            0,
        )))
        .unwrap();
        let to_tag = operation.call_method0(py, "tags").unwrap();
        let tags_op: &Vec<&str> = &Vec::extract(to_tag.as_ref(py)).unwrap();
        let tags_param: &[&str] = &["Operation", "Measurement", "MeasureQubit"];
        assert_eq!(tags_op, tags_param);
    })
}

/// Test hqslang function
#[test_case(Operation::from(MeasureQubit::new(0, String::from("ro"), 1)), String::from("MeasureQubit"); "MeasureQubit")]
#[test_case(Operation::from(PragmaGetStateVector::new(String::from("ro"), Some(create_circuit()))), String::from("PragmaGetStateVector"); "PragmaGetStateVector")]
#[test_case(Operation::from(PragmaGetDensityMatrix::new(String::from("ro"), Some(create_circuit()))), String::from("PragmaGetDensityMatrix"); "PragmaGetDensityMatrix")]
#[test_case(Operation::from(PragmaGetOccupationProbability::new(String::from("ro"), Some(create_circuit()))), String::from("PragmaGetOccupationProbability"); "PragmaGetOccupationProbability")]
#[test_case(Operation::from(PragmaGetPauliProduct::new(create_qubit_mapping(), String::from("ro"), create_circuit())), String::from("PragmaGetPauliProduct"); "PragmaGetPauliProduct")]
#[test_case(Operation::from(PragmaRepeatedMeasurement::new(String::from("ro"), 2, Some(create_qubit_mapping()))), String::from("PragmaRepeatedMeasurement"); "PragmaRepeatedMeasurement")]
fn test_pyo3_hqslang(input_measurement: Operation, hqslang_param: String) {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let operation = convert_operation_to_pyobject(input_measurement).unwrap();
        let hqslang_op: String =
            String::extract(operation.call_method0(py, "hqslang").unwrap().as_ref(py)).unwrap();
        assert_eq!(hqslang_op, hqslang_param);
    })
}

/// Test is_parametrized function
#[test_case(Operation::from(MeasureQubit::new(0, String::from("ro"), 1)); "MeasureQubit")]
#[test_case(Operation::from(PragmaGetStateVector::new(String::from("ro"), Some(create_circuit()))); "PragmaGetStateVector")]
#[test_case(Operation::from(PragmaGetDensityMatrix::new(String::from("ro"), Some(create_circuit()))); "PragmaGetDensityMatrix")]
#[test_case(Operation::from(PragmaGetOccupationProbability::new(String::from("ro"), Some(create_circuit()))); "PragmaGetOccupationProbability")]
#[test_case(Operation::from(PragmaGetPauliProduct::new(create_qubit_mapping(), String::from("ro"), create_circuit())); "PragmaGetPauliProduct")]
#[test_case(Operation::from(PragmaRepeatedMeasurement::new(String::from("ro"), 2, Some(create_qubit_mapping()))); "PragmaRepeatedMeasurement")]
fn test_pyo3_is_parametrized(input_measurement: Operation) {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let operation = convert_operation_to_pyobject(input_measurement).unwrap();
        assert!(!bool::extract(
            operation
                .call_method0(py, "is_parametrized")
                .unwrap()
                .as_ref(py)
        )
        .unwrap());
    })
}

/// Test substitute_parameters function
#[test_case(Operation::from(MeasureQubit::new(0, String::from("ro"), 0)); "MeasureQubit")]
#[test_case(Operation::from(PragmaGetStateVector::new(String::from("ro"), Some(create_circuit()))); "PragmaGetStateVector")]
#[test_case(Operation::from(PragmaGetDensityMatrix::new(String::from("ro"), Some(create_circuit()))); "PragmaGetDensityMatrix")]
#[test_case(Operation::from(PragmaGetOccupationProbability::new(String::from("ro"), Some(create_circuit()))); "PragmaGetOccupationProbability")]
#[test_case(Operation::from(PragmaGetPauliProduct::new(create_qubit_mapping(), String::from("ro"), create_circuit())); "PragmaGetPauliProduct")]
#[test_case(Operation::from(PragmaRepeatedMeasurement::new(String::from("ro"), 2, Some(create_qubit_mapping()))); "PragmaRepeatedMeasurement")]
fn test_pyo3_substitute_parameters(input_measurement: Operation) {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let operation = convert_operation_to_pyobject(input_measurement).unwrap();
        let mut substitution_dict: HashMap<&str, f64> = HashMap::new();
        substitution_dict.insert("ro", 1.0);
        let substitute_op = operation
            .call_method1(py, "substitute_parameters", (substitution_dict,))
            .unwrap();
        let substitute_param = operation;

        let comparison_copy = bool::extract(
            substitute_op
                .as_ref(py)
                .call_method1("__eq__", (substitute_param,))
                .unwrap(),
        )
        .unwrap();
        assert!(comparison_copy);

        let mut substitution_dict_error = HashMap::new();
        substitution_dict_error.insert("wrong", "fails");
        let comparison =
            substitute_op.call_method1(py, "substitute_parameters", (substitution_dict_error,));
        assert!(comparison.is_err());
    })
}

/// Test substitute_parameters() causing an error `not-a-real-number`
#[test_case(Operation::from(MeasureQubit::new(0, String::from("ro"), 0)); "MeasureQubit")]
#[test_case(Operation::from(PragmaGetStateVector::new(String::from("ro"), Some(create_circuit()))); "PragmaGetStateVector")]
#[test_case(Operation::from(PragmaGetDensityMatrix::new(String::from("ro"), Some(create_circuit()))); "PragmaGetDensityMatrix")]
#[test_case(Operation::from(PragmaGetOccupationProbability::new(String::from("ro"), Some(create_circuit()))); "PragmaGetOccupationProbability")]
#[test_case(Operation::from(PragmaGetPauliProduct::new(create_qubit_mapping(), String::from("ro"), create_circuit())); "PragmaGetPauliProduct")]
#[test_case(Operation::from(PragmaRepeatedMeasurement::new(String::from("ro"), 2, Some(create_qubit_mapping()))); "PragmaRepeatedMeasurement")]
fn test_pyo3_substitute_params_error(input_operation: Operation) {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let operation = convert_operation_to_pyobject(input_operation).unwrap();
        let mut substitution_dict: HashMap<&str, &str> = HashMap::new();
        substitution_dict.insert("ro", "test");
        let result = operation.call_method1(py, "substitute_parameters", (substitution_dict,));
        let result_ref = result.as_ref();
        assert!(result_ref.is_err());
    })
}

/// Test remap_qubits function
#[test_case(Operation::from(MeasureQubit::new(0, String::from("ro"), 0)),
            Operation::from(MeasureQubit::new(2, String::from("ro"), 0));
            "MeasureQubit")]
#[test_case(Operation::from(PragmaGetStateVector::new(String::from("ro"), Some(create_circuit()))),
            Operation::from(PragmaGetStateVector::new(String::from("ro"), Some(circuit_remapped())));
            "PragmaGetStateVector")]
#[test_case(Operation::from(PragmaGetDensityMatrix::new(String::from("ro"), Some(create_circuit()))),
            Operation::from(PragmaGetDensityMatrix::new(String::from("ro"), Some(circuit_remapped())));
            "PragmaGetDensityMatrix")]
#[test_case(Operation::from(PragmaGetOccupationProbability::new(String::from("ro"), Some(create_circuit()))),
            Operation::from(PragmaGetOccupationProbability::new(String::from("ro"), Some(circuit_remapped())));
            "PragmaGetOccupationProbability")]
#[test_case(Operation::from(PragmaGetPauliProduct::new(create_qubit_mapping(), String::from("ro"), create_circuit())),
            Operation::from(PragmaGetPauliProduct::new(qubits_remapped(), String::from("ro"), circuit_remapped()));
            "PragmaGetPauliProduct")]
#[test_case(Operation::from(PragmaRepeatedMeasurement::new(String::from("ro"), 2, Some(create_qubit_mapping()))),
            Operation::from(PragmaRepeatedMeasurement::new(String::from("ro"), 2, Some(qubits_remapped())));
            "PragmaRepeatedMeasurement")]
fn test_pyo3_remap_qubits(first_op: Operation, second_op: Operation) {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let operation = convert_operation_to_pyobject(first_op).unwrap();

        let remapped_op = operation
            .call_method1(py, "remap_qubits", (qubit_remapping(),))
            .unwrap();
        let comparison_op = convert_operation_to_pyobject(second_op).unwrap();

        let comparison = bool::extract(
            remapped_op
                .call_method1(py, "__eq__", (comparison_op,))
                .unwrap()
                .as_ref(py),
        )
        .unwrap();
        assert!(comparison);

        let mut qubit_mapping_error = HashMap::new();
        qubit_mapping_error.insert("fails", 3);
        let comparison = operation.call_method1(py, "remap_qubits", (qubit_mapping_error,));
        assert!(comparison.is_err());
    })
}

// test remap_qubits() function returning an error.
#[test_case(Operation::from(MeasureQubit::new(0, String::from("ro"), 1)); "MeasureQubit")]
fn test_pyo3_remapqubits_error(input_operation: Operation) {
    // preparation
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let operation = convert_operation_to_pyobject(input_operation).unwrap();
        // remap qubits
        let qubit_mapping: HashMap<usize, usize> = HashMap::new();
        let result = operation.call_method1(py, "remap_qubits", (qubit_mapping,));
        let result_ref = result.as_ref();
        assert!(result_ref.is_err());
    })
}

/// Test the __richcmp__ function
#[test_case(Operation::from(MeasureQubit::new(0, String::from("ro"), 0)),
            Operation::from(MeasureQubit::new(2, String::from("ro"), 0));
            "MeasureQubit")]
#[test_case(Operation::from(PragmaGetStateVector::new(String::from("ro"), Some(create_circuit()))),
            Operation::from(PragmaGetStateVector::new(String::from("ro"), Some(circuit_remapped())));
            "PragmaGetStateVector")]
#[test_case(Operation::from(PragmaGetDensityMatrix::new(String::from("ro"), Some(create_circuit()))),
            Operation::from(PragmaGetDensityMatrix::new(String::from("ro"), Some(circuit_remapped())));
            "PragmaGetDensityMatrix")]
#[test_case(Operation::from(PragmaGetOccupationProbability::new(String::from("ro"), Some(create_circuit()))),
            Operation::from(PragmaGetOccupationProbability::new(String::from("ro"), Some(circuit_remapped())));
            "PragmaGetOccupationProbability")]
#[test_case(Operation::from(PragmaGetPauliProduct::new(create_qubit_mapping(), String::from("ro"), create_circuit())),
            Operation::from(PragmaGetPauliProduct::new(qubits_remapped(), String::from("ro"), circuit_remapped()));
            "PragmaGetPauliProduct")]
#[test_case(Operation::from(PragmaRepeatedMeasurement::new(String::from("ro"), 2, Some(create_qubit_mapping()))),
            Operation::from(PragmaRepeatedMeasurement::new(String::from("ro"), 2, Some(qubits_remapped())));
            "PragmaRepeatedMeasurement")]
fn test_pyo3_richcmp(definition_1: Operation, definition_2: Operation) {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let operation_one = convert_operation_to_pyobject(definition_1).unwrap();
        let operation_two = convert_operation_to_pyobject(definition_2).unwrap();

        let comparison = bool::extract(
            operation_one
                .as_ref(py)
                .call_method1("__eq__", (operation_two.clone(),))
                .unwrap(),
        )
        .unwrap();
        assert!(!comparison);

        let comparison = bool::extract(
            operation_one
                .as_ref(py)
                .call_method1("__ne__", (operation_two.clone(),))
                .unwrap(),
        )
        .unwrap();
        assert!(comparison);

        let comparison = operation_one.call_method1(py, "__eq__", (vec!["fails"],));
        assert!(comparison.is_err());

        let comparison = operation_one.call_method1(py, "__ge__", (operation_two,));
        assert!(comparison.is_err());
    })
}

/// Test MeasureQubit new() function
#[test]
fn test_pyo3_new_set_number_of_measurements() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let operation = py.get_type::<MeasureQubitWrapper>();
        let new_op = operation
            .call1((0, "ro".to_string(), 1))
            .unwrap()
            .cast_as::<PyCell<MeasureQubitWrapper>>()
            .unwrap();

        let input_definition = Operation::from(MeasureQubit::new(0, String::from("ro"), 1));
        let copy_param = convert_operation_to_pyobject(input_definition).unwrap();
        let comparison_copy =
            bool::extract(new_op.call_method1("__eq__", (copy_param,)).unwrap()).unwrap();
        assert!(comparison_copy);

        let meas_wrapper = new_op.extract::<MeasureQubitWrapper>().unwrap();
        let new_op_diff = operation
            .call1((1, "ro".to_string(), 1))
            .unwrap()
            .cast_as::<PyCell<MeasureQubitWrapper>>()
            .unwrap();
        let meas_wrapper_diff = new_op_diff.extract::<MeasureQubitWrapper>().unwrap();
        let helper_ne: bool = meas_wrapper_diff != meas_wrapper;
        assert!(helper_ne);
        let helper_eq: bool = meas_wrapper == meas_wrapper.clone();
        assert!(helper_eq);

        assert_eq!(
            format!("{:?}", meas_wrapper),
            "MeasureQubitWrapper { internal: MeasureQubit { qubit: 0, readout: \"ro\", readout_index: 1 } }"
        );
    })
}

/// Test PragmaGetStateVector new() function
#[test]
fn test_pyo3_new_get_statevector() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let operation = py.get_type::<PragmaGetStateVectorWrapper>();
        let new_op = operation
            .call1(("ro".to_string(), Option::<CircuitWrapper>::None))
            .unwrap()
            .cast_as::<PyCell<PragmaGetStateVectorWrapper>>()
            .unwrap();

        let input_definition = Operation::from(PragmaGetStateVector::new(String::from("ro"), None));
        let copy_param = convert_operation_to_pyobject(input_definition).unwrap();
        let comparison_copy =
            bool::extract(new_op.call_method1("__eq__", (copy_param,)).unwrap()).unwrap();
        assert!(comparison_copy);

        let meas_wrapper = new_op.extract::<PragmaGetStateVectorWrapper>().unwrap();
        let new_op_diff = operation
            .call1(("ro2".to_string(), new_circuit(py)))
            .unwrap()
            .cast_as::<PyCell<PragmaGetStateVectorWrapper>>()
            .unwrap();
        let meas_wrapper_diff = new_op_diff
            .extract::<PragmaGetStateVectorWrapper>()
            .unwrap();
        let helper_ne: bool = meas_wrapper_diff != meas_wrapper;
        assert!(helper_ne);
        let helper_eq: bool = meas_wrapper == meas_wrapper.clone();
        assert!(helper_eq);

        assert_eq!(
            format!("{:?}", meas_wrapper),
            "PragmaGetStateVectorWrapper { internal: PragmaGetStateVector { readout: \"ro\", circuit: None } }"
        );
    })
}

/// Test PragmaGetDensityMatrix new() function
#[test]
fn test_pyo3_new_get_density_matrix() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let operation = py.get_type::<PragmaGetDensityMatrixWrapper>();
        let new_op = operation
            .call1(("ro".to_string(), Option::<CircuitWrapper>::None))
            .unwrap()
            .cast_as::<PyCell<PragmaGetDensityMatrixWrapper>>()
            .unwrap();

        let input_definition =
            Operation::from(PragmaGetDensityMatrix::new(String::from("ro"), None));
        let copy_param = convert_operation_to_pyobject(input_definition).unwrap();
        let comparison_copy =
            bool::extract(new_op.call_method1("__eq__", (copy_param,)).unwrap()).unwrap();
        assert!(comparison_copy);

        let meas_wrapper = new_op.extract::<PragmaGetDensityMatrixWrapper>().unwrap();
        let new_op_diff = operation
            .call1(("ro2".to_string(), new_circuit(py)))
            .unwrap()
            .cast_as::<PyCell<PragmaGetDensityMatrixWrapper>>()
            .unwrap();
        let meas_wrapper_diff = new_op_diff
            .extract::<PragmaGetDensityMatrixWrapper>()
            .unwrap();
        let helper_ne: bool = meas_wrapper_diff != meas_wrapper;
        assert!(helper_ne);
        let helper_eq: bool = meas_wrapper == meas_wrapper.clone();
        assert!(helper_eq);

        assert_eq!(
            format!("{:?}", meas_wrapper),
            "PragmaGetDensityMatrixWrapper { internal: PragmaGetDensityMatrix { readout: \"ro\", circuit: None } }"
        );
    })
}

/// Test PragmaGetOccupationProbability new() function
#[test]
fn test_pyo3_new_get_occupation_proba() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let operation = py.get_type::<PragmaGetOccupationProbabilityWrapper>();
        let new_op = operation
            .call1(("ro".to_string(), Option::<CircuitWrapper>::None))
            .unwrap()
            .cast_as::<PyCell<PragmaGetOccupationProbabilityWrapper>>()
            .unwrap();

        let input_definition = Operation::from(PragmaGetOccupationProbability::new(
            String::from("ro"),
            None,
        ));
        let copy_param = convert_operation_to_pyobject(input_definition).unwrap();
        let comparison_copy =
            bool::extract(new_op.call_method1("__eq__", (copy_param,)).unwrap()).unwrap();
        assert!(comparison_copy);

        let meas_wrapper = new_op
            .extract::<PragmaGetOccupationProbabilityWrapper>()
            .unwrap();
        let new_op_diff = operation
            .call1(("ro2".to_string(), new_circuit(py)))
            .unwrap()
            .cast_as::<PyCell<PragmaGetOccupationProbabilityWrapper>>()
            .unwrap();
        let meas_wrapper_diff = new_op_diff
            .extract::<PragmaGetOccupationProbabilityWrapper>()
            .unwrap();
        let helper_ne: bool = meas_wrapper_diff != meas_wrapper;
        assert!(helper_ne);
        let helper_eq: bool = meas_wrapper == meas_wrapper.clone();
        assert!(helper_eq);

        assert_eq!(
            format!("{:?}", meas_wrapper),
            "PragmaGetOccupationProbabilityWrapper { internal: PragmaGetOccupationProbability { readout: \"ro\", circuit: None } }"
        );
    })
}

/// Test PragmaGetPauliProduct new() function
#[test]
fn test_pyo3_new_get_pauli_product() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let operation = py.get_type::<PragmaGetPauliProductWrapper>();
        let new_op = operation
            .call1((create_qubit_mapping(), "ro".to_string(), new_circuit(py)))
            .unwrap()
            .cast_as::<PyCell<PragmaGetPauliProductWrapper>>()
            .unwrap();

        let input_definition = Operation::from(PragmaGetPauliProduct::new(
            create_qubit_mapping(),
            String::from("ro"),
            Circuit::default(),
        ));
        let copy_param = convert_operation_to_pyobject(input_definition).unwrap();
        let comparison_copy =
            bool::extract(new_op.call_method1("__eq__", (copy_param,)).unwrap()).unwrap();
        assert!(comparison_copy);

        let meas_wrapper = new_op.extract::<PragmaGetPauliProductWrapper>().unwrap();
        let new_op_diff = operation
            .call1((create_qubit_mapping(), "ro2".to_string(), new_circuit(py)))
            .unwrap()
            .cast_as::<PyCell<PragmaGetPauliProductWrapper>>()
            .unwrap();
        let meas_wrapper_diff = new_op_diff
            .extract::<PragmaGetPauliProductWrapper>()
            .unwrap();
        let helper_ne: bool = meas_wrapper_diff != meas_wrapper;
        assert!(helper_ne);
        let helper_eq: bool = meas_wrapper == meas_wrapper.clone();
        assert!(helper_eq);

        assert_eq!(
            format!("{:?}", meas_wrapper),
            "PragmaGetPauliProductWrapper { internal: PragmaGetPauliProduct { qubit_paulis: {0: 1}, readout: \"ro\", circuit: Circuit { definitions: [], operations: [], _roqoqo_version: RoqoqoVersion } } }"
        );
    })
}

/// Test PragmaRepeatedMeasurement new() function
#[test]
fn test_pyo3_new_repeated_measurement() {
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        let operation = py.get_type::<PragmaRepeatedMeasurementWrapper>();
        let new_op = operation
            .call1(("ro".to_string(), 1, Some(create_qubit_mapping())))
            .unwrap()
            .cast_as::<PyCell<PragmaRepeatedMeasurementWrapper>>()
            .unwrap();

        let input_definition = Operation::from(PragmaRepeatedMeasurement::new(
            String::from("ro"),
            1,
            Some(create_qubit_mapping()),
        ));
        let copy_param = convert_operation_to_pyobject(input_definition).unwrap();
        let comparison_copy =
            bool::extract(new_op.call_method1("__eq__", (copy_param,)).unwrap()).unwrap();
        assert!(comparison_copy);

        let meas_wrapper = new_op
            .extract::<PragmaRepeatedMeasurementWrapper>()
            .unwrap();
        let new_op_diff = operation
            .call1(("ro".to_string(), 2, Some(create_qubit_mapping())))
            .unwrap()
            .cast_as::<PyCell<PragmaRepeatedMeasurementWrapper>>()
            .unwrap();
        let meas_wrapper_diff = new_op_diff
            .extract::<PragmaRepeatedMeasurementWrapper>()
            .unwrap();
        let helper_ne: bool = meas_wrapper_diff != meas_wrapper;
        assert!(helper_ne);
        let helper_eq: bool = meas_wrapper == meas_wrapper.clone();
        assert!(helper_eq);

        assert_eq!(
            format!("{:?}", meas_wrapper),
            "PragmaRepeatedMeasurementWrapper { internal: PragmaRepeatedMeasurement { readout: \"ro\", number_measurements: 1, qubit_mapping: Some({0: 1}) } }"
        );
    })
}
