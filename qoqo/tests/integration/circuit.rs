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

use pyo3::exceptions::PyIndexError;
use pyo3::prelude::*;
use qoqo::operations::{
    convert_operation_to_pyobject, PragmaOverrotationWrapper, RotateXWrapper, RotateYWrapper,
};
use qoqo::{CircuitWrapper, OperationIteratorWrapper, QOQO_VERSION};
use qoqo_calculator::CalculatorFloat;
use roqoqo::operations::Operation;
use roqoqo::operations::*;
use roqoqo::ROQOQO_VERSION;
use std::collections::{HashMap, HashSet};
use test_case::test_case;

// helper functions
fn new_circuit(py: Python) -> &PyCell<CircuitWrapper> {
    let circuit_type = py.get_type::<CircuitWrapper>();
    circuit_type
        .call0()
        .unwrap()
        .cast_as::<PyCell<CircuitWrapper>>()
        .unwrap()
}

fn populate_circuit_rotatex(
    py: Python,
    circuit: &PyCell<CircuitWrapper>,
    start: usize,
    stop: usize,
) {
    let rotatex_type = py.get_type::<RotateXWrapper>();
    for i in start..stop {
        let new_rotatex_0 = rotatex_type.call1((i, i)).unwrap();
        // .cast_as::<PyCell<RotateXWrapper>>()
        // .unwrap();
        circuit.call_method1("add", (new_rotatex_0,)).unwrap();
    }
}

/// Test default function of CircuitWrapper
#[test]
fn test_default() {
    let operation = convert_operation_to_pyobject(Operation::from(PauliX::new(0))).unwrap();
    let gil = pyo3::Python::acquire_gil();
    let py = gil.python();
    let circuit = new_circuit(py);
    circuit.call_method1("add", (operation.clone(),)).unwrap();
    let circuit_wrapper = circuit.extract::<CircuitWrapper>();

    let helper_ne: bool = CircuitWrapper::default() != circuit_wrapper.unwrap();
    assert!(helper_ne);
    let helper_eq: bool = CircuitWrapper::default() == CircuitWrapper::new();
    assert!(helper_eq);

    assert_eq!(
        format!("{:?}", CircuitWrapper::new()),
        "CircuitWrapper { internal: Circuit { definitions: [], operations: [] } }"
    );
}

/// Test substitute_parameters function of Circuit
#[test]
fn test_substitute_parameters() {
    let added_operation = Operation::from(RotateX::new(0, CalculatorFloat::from("test")));
    let gil = pyo3::Python::acquire_gil();
    let py = gil.python();
    let operation = convert_operation_to_pyobject(added_operation).unwrap();
    let circuit = new_circuit(py);
    circuit.call_method1("add", (operation.clone(),)).unwrap();

    let mut substitution_dict: HashMap<&str, f64> = HashMap::new();
    substitution_dict.insert("test", 1.0);
    let substitute_circ = circuit
        .call_method1("substitute_parameters", (substitution_dict,))
        .unwrap();

    let to_sub = Operation::from(RotateX::new(0, CalculatorFloat::from(1.0)));
    let subbed_operation = convert_operation_to_pyobject(to_sub).unwrap();

    let comp_op = substitute_circ.call_method1("__getitem__", (0,)).unwrap();
    let comparison =
        bool::extract(comp_op.call_method1("__eq__", (subbed_operation,)).unwrap()).unwrap();
    assert!(comparison);

    let mut substitution_dict_error = HashMap::new();
    substitution_dict_error.insert("fails", 0.0);
    let comparison = circuit.call_method1("substitute_parameters", (substitution_dict_error,));
    assert!(comparison.is_err());
}

/// Test remap_qubits function of Circuit
#[test]
fn test_remap_qubits() {
    let added_operation = Operation::from(RotateX::new(0, CalculatorFloat::from(1.0)));
    let gil = pyo3::Python::acquire_gil();
    let py = gil.python();
    let operation = convert_operation_to_pyobject(added_operation).unwrap();
    let circuit = new_circuit(py);
    circuit.call_method1("add", (operation.clone(),)).unwrap();

    let mut qubit_mapping: HashMap<usize, usize> = HashMap::new();
    qubit_mapping.insert(0, 2);
    let remap_circ = circuit
        .call_method1("remap_qubits", (qubit_mapping,))
        .unwrap();

    let to_remap = Operation::from(RotateX::new(2, CalculatorFloat::from(1.0)));
    let remapped_operation = convert_operation_to_pyobject(to_remap).unwrap();

    let comp_op = remap_circ.call_method1("__getitem__", (0,)).unwrap();
    let comparison = bool::extract(
        comp_op
            .call_method1("__eq__", (remapped_operation,))
            .unwrap(),
    )
    .unwrap();
    assert!(comparison);

    let mut qubit_mapping_error = HashMap::new();
    qubit_mapping_error.insert(5, 3);
    let comparison = circuit.call_method1("remap_qubits", (qubit_mapping_error,));
    assert!(comparison.is_err());
}

/// Test count_occurences function of Circuit
#[test]
fn test_count_occurences() {
    let added_op1 = Operation::from(DefinitionBit::new("ro".to_string(), 1, false));
    let added_op2 = Operation::from(RotateX::new(0, CalculatorFloat::from(1.0)));
    let added_op3 = Operation::from(PauliX::new(0));
    let gil = pyo3::Python::acquire_gil();
    let py = gil.python();
    let operation1 = convert_operation_to_pyobject(added_op1).unwrap();
    let operation2 = convert_operation_to_pyobject(added_op2).unwrap();
    let operation3 = convert_operation_to_pyobject(added_op3).unwrap();
    let circuit = new_circuit(py);
    circuit.call_method1("add", (operation1.clone(),)).unwrap();
    circuit.call_method1("add", (operation2.clone(),)).unwrap();
    circuit.call_method1("add", (operation3.clone(),)).unwrap();

    let comp_op = usize::extract(
        circuit
            .call_method1("count_occurences", (vec!["Definition"],))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(comp_op, 1_usize);
    let comp_op = usize::extract(
        circuit
            .call_method1("count_occurences", (vec!["Operation"],))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(comp_op, 3_usize);
    let comp_op = usize::extract(
        circuit
            .call_method1("count_occurences", (vec!["RotateX"],))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(comp_op, 1_usize);
    let comp_op = usize::extract(
        circuit
            .call_method1("count_occurences", (vec!["SingleQubitGateOperation"],))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(comp_op, 2_usize);
    let comp_op = usize::extract(
        circuit
            .call_method1("count_occurences", (vec!["MadeUp"],))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(comp_op, 0_usize);
}

/// Test get_operation_types function of Circuit
#[test]
fn test_get_operation_types() {
    let added_op1 = Operation::from(DefinitionBit::new("ro".to_string(), 1, false));
    let added_op2 = Operation::from(RotateX::new(0, CalculatorFloat::from(1.0)));
    let added_op3 = Operation::from(PauliX::new(0));
    let gil = pyo3::Python::acquire_gil();
    let py = gil.python();
    let operation1 = convert_operation_to_pyobject(added_op1).unwrap();
    let operation2 = convert_operation_to_pyobject(added_op2).unwrap();
    let operation3 = convert_operation_to_pyobject(added_op3).unwrap();
    let circuit = new_circuit(py);
    circuit.call_method1("add", (operation1.clone(),)).unwrap();
    circuit.call_method1("add", (operation2.clone(),)).unwrap();
    circuit.call_method1("add", (operation3.clone(),)).unwrap();

    let mut op_types: HashSet<&str> = HashSet::new();
    op_types.insert("DefinitionBit");
    op_types.insert("RotateX");
    op_types.insert("PauliX");

    let comp_op = HashSet::extract(circuit.call_method0("get_operation_types").unwrap()).unwrap();
    assert_eq!(comp_op, op_types);
}

/// Test copy and deepcopy functions of Circuit
#[test]
fn test_copy_deepcopy() {
    let gil = pyo3::Python::acquire_gil();
    let py = gil.python();
    let circuit = new_circuit(py);
    populate_circuit_rotatex(py, circuit, 0, 3);

    let copy_circ = circuit.call_method0("__copy__").unwrap();
    let deepcopy_circ = circuit.call_method1("__deepcopy__", ("",)).unwrap();
    let copy_deepcopy_param = circuit.clone();

    let comparison_copy = bool::extract(
        copy_circ
            .call_method1("__eq__", (copy_deepcopy_param.clone(),))
            .unwrap(),
    )
    .unwrap();
    assert!(comparison_copy);
    let comparison_deepcopy = bool::extract(
        deepcopy_circ
            .call_method1("__eq__", (copy_deepcopy_param,))
            .unwrap(),
    )
    .unwrap();
    assert!(comparison_deepcopy);
}

/// Test qoqo_versions function of Circuit
#[test]
fn test_qoqo_versions() {
    let gil = pyo3::Python::acquire_gil();
    let py = gil.python();
    let circuit = new_circuit(py);
    populate_circuit_rotatex(py, circuit, 0, 3);

    let comparison_copy: Vec<&str> =
        Vec::extract(circuit.call_method0("_qoqo_versions").unwrap()).unwrap();
    assert_eq!(comparison_copy, vec![ROQOQO_VERSION, QOQO_VERSION]);
}

/// Test to_ and from_bincode functions of Circuit
#[test]
fn test_to_from_bincode() {
    let gil = pyo3::Python::acquire_gil();
    let py = gil.python();
    let circuit = new_circuit(py);
    populate_circuit_rotatex(py, circuit, 0, 3);

    let serialised = circuit.call_method0("to_bincode").unwrap();
    let new = new_circuit(py);
    let deserialised = new.call_method1("from_bincode", (serialised,)).unwrap();

    let deserialised_error =
        new.call_method1("from_bincode", (bincode::serialize("fails").unwrap(),));
    assert!(deserialised_error.is_err());

    let deserialised_error =
        new.call_method1("from_bincode", (bincode::serialize(&vec![0]).unwrap(),));
    assert!(deserialised_error.is_err());

    let comparison =
        bool::extract(deserialised.call_method1("__eq__", (circuit,)).unwrap()).unwrap();
    assert!(comparison)
}

/// Test to_ and from_json functions of Circuit
#[test]
fn test_to_from_json() {
    let gil = pyo3::Python::acquire_gil();
    let py = gil.python();
    let circuit = new_circuit(py);
    populate_circuit_rotatex(py, circuit, 0, 3);

    let serialised = circuit.call_method0("to_json").unwrap();
    let new = new_circuit(py);
    let deserialised = new.call_method1("from_json", (serialised,)).unwrap();

    let deserialised_error =
        new.call_method1("from_json", (serde_json::to_string("fails").unwrap(),));
    assert!(deserialised_error.is_err());

    let deserialised_error =
        new.call_method1("from_json", (serde_json::to_string(&vec![0]).unwrap(),));
    assert!(deserialised_error.is_err());

    let serialised_error = serialised.call_method0("to_json");
    assert!(serialised_error.is_err());

    let comparison =
        bool::extract(deserialised.call_method1("__eq__", (circuit,)).unwrap()).unwrap();
    assert!(comparison)
}

///  Test single index set and write access using "get" function
#[test]
fn test_single_index_access_get() {
    let gil = pyo3::Python::acquire_gil();
    let py = gil.python();
    let circuit = new_circuit(py);
    populate_circuit_rotatex(py, circuit, 0, 3);

    // test access at index 1
    let comp_op = circuit.call_method1("get", (1,)).unwrap();
    let operation =
        convert_operation_to_pyobject(Operation::from(RotateX::new(1, CalculatorFloat::from(1))))
            .unwrap();
    let comparison = bool::extract(comp_op.call_method1("__eq__", (operation,)).unwrap()).unwrap();
    assert!(comparison);

    // test setting new operation at index 1
    let operation2 =
        convert_operation_to_pyobject(Operation::from(RotateX::new(1, CalculatorFloat::from(10))))
            .unwrap();

    circuit
        .call_method1("__setitem__", (1, operation2.clone()))
        .unwrap();

    let comp_op = circuit.call_method1("get", (1,)).unwrap();
    let comparison = bool::extract(
        comp_op
            .call_method1("__eq__", (operation2.clone(),))
            .unwrap(),
    )
    .unwrap();
    assert!(comparison);

    let comparison = circuit.call_method1("get", (20,));
    assert!(comparison.is_err());
}

/// Test get_slice property of Circuit
#[test]
fn test_get_slice() {
    let gil = pyo3::Python::acquire_gil();
    let py = gil.python();
    let circuit = new_circuit(py);
    populate_circuit_rotatex(py, circuit, 0, 4);

    let circuit2 = new_circuit(py);
    populate_circuit_rotatex(py, circuit2, 1, 3);

    let circuit3 = new_circuit(py);
    populate_circuit_rotatex(py, circuit3, 0, 3);

    let circuit4 = new_circuit(py);
    populate_circuit_rotatex(py, circuit4, 2, 4);

    let new_circuit_slice = circuit.call_method1("get_slice", (1, 2)).unwrap();
    let comparison = bool::extract(
        new_circuit_slice
            .call_method1("__eq__", (circuit2,))
            .unwrap(),
    )
    .unwrap();
    assert!(comparison);

    let new_circuit_slice = circuit
        .call_method1("get_slice", (Option::<usize>::None, 2))
        .unwrap();
    let comparison = bool::extract(
        new_circuit_slice
            .call_method1("__eq__", (circuit3,))
            .unwrap(),
    )
    .unwrap();
    assert!(comparison);

    let new_circuit_slice = circuit
        .call_method1("get_slice", (2, Option::<usize>::None))
        .unwrap();
    let comparison = bool::extract(
        new_circuit_slice
            .call_method1("__eq__", (circuit4,))
            .unwrap(),
    )
    .unwrap();
    assert!(comparison);

    match circuit.call_method1("get_slice", (1, 20)) {
        Err(x) => assert!(x.is_instance::<PyIndexError>(py)),
        _ => panic!("Wrong error"),
    }

    match circuit.call_method1("get_slice", (2, 1)) {
        Err(x) => assert!(x.is_instance::<PyIndexError>(py)),
        _ => panic!("Wrong error"),
    }

    match circuit.call_method1("get_slice", (21, 22)) {
        Err(x) => assert!(x.is_instance::<PyIndexError>(py)),
        _ => panic!("Wrong error"),
    }
}

/// Test definitions function of Circuit
#[test]
fn test_definitions() {
    let added_op1 = Operation::from(DefinitionBit::new("ro".to_string(), 1, false));
    let added_op2 = Operation::from(InputSymbolic::new("test".to_string(), 1.0));
    let gil = pyo3::Python::acquire_gil();
    let py = gil.python();
    let operation1 = convert_operation_to_pyobject(added_op1).unwrap();
    let operation2 = convert_operation_to_pyobject(added_op2).unwrap();
    let circuit = new_circuit(py);
    circuit.call_method1("add", (operation1.clone(),)).unwrap();
    circuit.call_method1("add", (operation2.clone(),)).unwrap();

    let comp_op = circuit.call_method0("definitions").unwrap();
    let comparison = bool::extract(
        comp_op
            .call_method1("__eq__", (vec![operation1, operation2],))
            .unwrap(),
    )
    .unwrap();
    assert!(comparison)
}

/// Test filter_by_tag function of Circuit
#[test]
fn test_filter_by_tag() {
    let added_op1 = Operation::from(DefinitionBit::new("ro".to_string(), 1, false));
    let added_op2 = Operation::from(InputSymbolic::new("test".to_string(), 1.0));
    let gil = pyo3::Python::acquire_gil();
    let py = gil.python();
    let operation1 = convert_operation_to_pyobject(added_op1).unwrap();
    let operation2 = convert_operation_to_pyobject(added_op2).unwrap();
    let circuit = new_circuit(py);
    circuit.call_method1("add", (operation1.clone(),)).unwrap();
    circuit.call_method1("add", (operation2.clone(),)).unwrap();
    populate_circuit_rotatex(py, circuit, 0, 2);

    let comp_op = circuit
        .call_method1("filter_by_tag", ("Definition",))
        .unwrap();
    let comparison = bool::extract(
        comp_op
            .call_method1("__eq__", (vec![operation1, operation2],))
            .unwrap(),
    )
    .unwrap();
    assert!(comparison);

    let rotatex_type = py.get_type::<RotateXWrapper>();
    let rotatex_0 = rotatex_type
        .call1((0, 0))
        .unwrap()
        .cast_as::<PyCell<RotateXWrapper>>()
        .unwrap();
    let rotatex_1 = rotatex_type
        .call1((1, 1))
        .unwrap()
        .cast_as::<PyCell<RotateXWrapper>>()
        .unwrap();

    let comp_op = circuit.call_method1("filter_by_tag", ("RotateX",)).unwrap();
    let comparison = bool::extract(
        comp_op
            .call_method1("__eq__", (vec![rotatex_0, rotatex_1],))
            .unwrap(),
    )
    .unwrap();
    assert!(comparison)
}

/// Test add function
#[test_case(Operation::from(RotateX::new(0, CalculatorFloat::from(0))); "RotateX float")]
#[test_case(Operation::from(RotateZ::new(1, CalculatorFloat::from(1.3))); "RotateZ float")]
#[test_case(Operation::from(SingleQubitGate::new(2, CalculatorFloat::from(0), CalculatorFloat::from("var"), CalculatorFloat::from(0), CalculatorFloat::from(0), CalculatorFloat::from(0), )); "SingleQubitGate float")]
fn test_circuit_add_function(added_operation: Operation) {
    let gil = pyo3::Python::acquire_gil();
    let py = gil.python();
    let operation = convert_operation_to_pyobject(added_operation).unwrap();
    let circuit = new_circuit(py);
    circuit.call_method1("add", (operation.clone(),)).unwrap();

    let comp_op = circuit.call_method1("__getitem__", (0,)).unwrap();
    let comparison = bool::extract(comp_op.call_method1("__eq__", (operation,)).unwrap()).unwrap();
    assert!(comparison);

    let comparison = circuit.call_method1("add", (vec!["fails"],));
    assert!(comparison.is_err());
}

/// Test the __repr__ and __format__ functions
#[test]
fn test_format_repr() {
    let gil = pyo3::Python::acquire_gil();
    let py = gil.python();
    let circuit = new_circuit(py);
    populate_circuit_rotatex(py, circuit, 0, 2);
    let format_repr = "RotateX(RotateX { qubit: 0, theta: Float(0.0) })\nRotateX(RotateX { qubit: 1, theta: Float(1.0) })\n";

    let to_format = circuit.call_method1("__format__", ("",)).unwrap();
    let format_op: &str = <&str>::extract(to_format).unwrap();

    let to_repr = circuit.call_method0("__repr__").unwrap();
    let repr_op: &str = <&str>::extract(to_repr).unwrap();

    assert_eq!(format_op, format_repr);
    assert_eq!(repr_op, format_repr);
}

/// Test fmt::Debug for OperationIteratorWrapper
#[test]
fn test_fmt_circuititerator() {
    let gil = pyo3::Python::acquire_gil();
    let py = gil.python();
    let new_circuit = new_circuit(py);
    populate_circuit_rotatex(py, new_circuit, 0, 2);
    let circuit_iter = new_circuit
        .call_method0("__iter__")
        .unwrap()
        .cast_as::<PyCell<OperationIteratorWrapper>>()
        .unwrap();

    let fmt = "RefCell { value: OperationIteratorWrapper { internal: OperationIterator { definition_iter: IntoIter([]), operation_iter: IntoIter([RotateX(RotateX { qubit: 0, theta: Float(0.0) }), RotateX(RotateX { qubit: 1, theta: Float(1.0) })]) } } }";

    assert_eq!(format!("{:?}", circuit_iter), fmt);
}

/// Test the __richcmp__ function
#[test]
fn test_richcmp() {
    let gil = pyo3::Python::acquire_gil();
    let py = gil.python();
    let circuit_one = new_circuit(py);
    populate_circuit_rotatex(py, circuit_one, 0, 2);
    let circuit_two = new_circuit(py);
    populate_circuit_rotatex(py, circuit_two, 0, 3);
    let operation1 = convert_operation_to_pyobject(Operation::from(PauliX::new(0))).unwrap();

    let comparison =
        bool::extract(circuit_one.call_method1("__eq__", (circuit_two,)).unwrap()).unwrap();
    assert!(!comparison);
    let comparison = bool::extract(
        circuit_one
            .call_method1("__eq__", (operation1.clone(),))
            .unwrap(),
    )
    .unwrap();
    assert!(!comparison);

    let comparison =
        bool::extract(circuit_one.call_method1("__ne__", (circuit_two,)).unwrap()).unwrap();
    assert!(comparison);
    let comparison = bool::extract(
        circuit_one
            .call_method1("__ne__", (operation1.clone(),))
            .unwrap(),
    )
    .unwrap();
    assert!(comparison);

    let comparison = circuit_one.call_method1("__ge__", (operation1,));
    assert!(comparison.is_err());
}

#[test]
fn test_circuit_iadd_magic_method() {
    let added_op1 = Operation::from(DefinitionBit::new("ro".to_string(), 1, false));
    let added_op2 = Operation::from(RotateX::new(0, CalculatorFloat::from(1.0)));
    let added_op3 = Operation::from(PauliX::new(0));
    let gil = pyo3::Python::acquire_gil();
    let py = gil.python();
    let operation1 = convert_operation_to_pyobject(added_op1).unwrap();
    let operation2 = convert_operation_to_pyobject(added_op2).unwrap();
    let operation3 = convert_operation_to_pyobject(added_op3).unwrap();

    let added_circuit = new_circuit(py);
    added_circuit
        .call_method1("add", (operation3.clone(),))
        .unwrap();

    let circuit = new_circuit(py);
    circuit.call_method1("add", (operation1.clone(),)).unwrap();
    circuit
        .call_method1("__iadd__", (operation2.clone(),))
        .unwrap();
    circuit
        .call_method1("__iadd__", (added_circuit.clone(),))
        .unwrap();

    let comp_op = circuit.call_method1("__getitem__", (0,)).unwrap();
    let comparison = bool::extract(comp_op.call_method1("__eq__", (operation1,)).unwrap()).unwrap();
    assert!(comparison);

    let comp_op = circuit.call_method1("__getitem__", (1,)).unwrap();
    let comparison = bool::extract(comp_op.call_method1("__eq__", (operation2,)).unwrap()).unwrap();
    assert!(comparison);

    let comp_op = circuit.call_method1("__getitem__", (2,)).unwrap();
    let comparison = bool::extract(comp_op.call_method1("__eq__", (operation3,)).unwrap()).unwrap();
    assert!(comparison);

    let comparison = circuit.call_method1("__iadd__", (vec!["fails"],));
    assert!(comparison.is_err());
}

#[test]
fn test_circuit_add_magic_method() {
    let added_op1 = Operation::from(DefinitionBit::new("ro".to_string(), 1, false));
    let added_op2 = Operation::from(RotateX::new(0, CalculatorFloat::from(1.0)));
    let added_op3 = Operation::from(PauliX::new(0));
    let gil = pyo3::Python::acquire_gil();
    let py = gil.python();
    let operation1 = convert_operation_to_pyobject(added_op1).unwrap();
    let operation2 = convert_operation_to_pyobject(added_op2).unwrap();
    let operation3 = convert_operation_to_pyobject(added_op3).unwrap();

    let added_circuit = new_circuit(py);
    added_circuit
        .call_method1("add", (operation3.clone(),))
        .unwrap();

    let circuit = new_circuit(py);
    circuit.call_method1("add", (operation1.clone(),)).unwrap();
    let circuit1 = circuit
        .call_method1("__add__", (operation2.clone(),))
        .unwrap();
    let circuit2 = circuit1
        .call_method1("__add__", (added_circuit.clone(),))
        .unwrap();

    let comp_op = circuit2.call_method1("__getitem__", (0,)).unwrap();
    let comparison = bool::extract(comp_op.call_method1("__eq__", (operation1,)).unwrap()).unwrap();
    assert!(comparison);

    let comp_op = circuit2.call_method1("__getitem__", (1,)).unwrap();
    let comparison = bool::extract(comp_op.call_method1("__eq__", (operation2,)).unwrap()).unwrap();
    assert!(comparison);

    let comp_op = circuit2.call_method1("__getitem__", (2,)).unwrap();
    let comparison = bool::extract(comp_op.call_method1("__eq__", (operation3,)).unwrap()).unwrap();
    assert!(comparison);

    let comparison = circuit.call_method1("__add__", (vec!["fails"],));
    assert!(comparison.is_err());
}

/// Test iterator interface of Circuit
#[test]
fn test_iter() {
    let gil = pyo3::Python::acquire_gil();
    let py = gil.python();
    let new_circuit = new_circuit(py);
    populate_circuit_rotatex(py, new_circuit, 0, 3);

    let rotatex_type = py.get_type::<RotateXWrapper>();
    let new_rotatex_0 = rotatex_type
        .call1((0, 0))
        .unwrap()
        .cast_as::<PyCell<RotateXWrapper>>()
        .unwrap();
    let new_rotatex_1 = rotatex_type
        .call1((1, 1))
        .unwrap()
        .cast_as::<PyCell<RotateXWrapper>>()
        .unwrap();
    let new_rotatex_2 = rotatex_type
        .call1((2, 2))
        .unwrap()
        .cast_as::<PyCell<RotateXWrapper>>()
        .unwrap();
    let comparison_vec = vec![new_rotatex_0, new_rotatex_1, new_rotatex_2];

    let t = new_circuit
        .call_method0("__iter__")
        .unwrap()
        .cast_as::<PyCell<OperationIteratorWrapper>>()
        .unwrap();

    for i in 0..3 {
        let comp_op = t.call_method0("__next__").unwrap();
        let comparison = bool::extract(
            comp_op
                .call_method1("__eq__", (comparison_vec[i],))
                .unwrap(),
        )
        .unwrap();
        assert!(comparison)
    }

    let iter_op = t.call_method0("__iter__").unwrap();
    let comparison = bool::extract(iter_op.call_method1("__eq__", (t,)).unwrap()).unwrap();
    assert!(comparison);
}

/// Test the __len__ function
#[test]
fn test_len() {
    let gil = pyo3::Python::acquire_gil();
    let py = gil.python();
    let circuit = new_circuit(py);
    populate_circuit_rotatex(py, circuit, 0, 5);

    let len_op: usize = usize::extract(circuit.call_method0("__len__").unwrap()).unwrap();
    assert_eq!(len_op, 5_usize);
}

///  Test single index set and write access using "__getitem__" function
#[test]
fn test_single_index_access_getitem() {
    let gil = pyo3::Python::acquire_gil();
    let py = gil.python();
    let circuit = new_circuit(py);
    populate_circuit_rotatex(py, circuit, 0, 3);

    // test access at index 1
    let comp_op = circuit.call_method1("__getitem__", (1,)).unwrap();
    let operation =
        convert_operation_to_pyobject(Operation::from(RotateX::new(1, CalculatorFloat::from(1))))
            .unwrap();
    let comparison = bool::extract(comp_op.call_method1("__eq__", (operation,)).unwrap()).unwrap();
    assert!(comparison);

    // test setting new operation at index 1
    let operation2 =
        convert_operation_to_pyobject(Operation::from(RotateX::new(1, CalculatorFloat::from(10))))
            .unwrap();

    circuit
        .call_method1("__setitem__", (1, operation2.clone()))
        .unwrap();

    let comp_op = circuit.call_method1("__getitem__", (1,)).unwrap();
    let comparison = bool::extract(
        comp_op
            .call_method1("__eq__", (operation2.clone(),))
            .unwrap(),
    )
    .unwrap();
    assert!(comparison);

    let comparison = circuit.call_method1("__setitem__", (1, vec!["fails"]));
    assert!(comparison.is_err());

    let comparison = circuit.call_method1("__getitem__", (20,));
    assert!(comparison.is_err());

    let comparison = circuit.call_method1("__setitem__", (3, operation2));
    assert!(comparison.is_err());
}

#[test]
fn test_convert_into_circuit() {
    let added_op = Operation::from(PauliX::new(0));
    let gil = pyo3::Python::acquire_gil();
    let py = gil.python();
    let operation = convert_operation_to_pyobject(added_op).unwrap();

    let added_circuit = new_circuit(py);
    let comparison = added_circuit.call_method1("convert_into_circuit", (operation,));
    assert!(comparison.is_err());
}

/// Test function overrotate() for Circuit
#[test]
// #[cfg(feature = "overrotate")]
fn test_circuit_overrotate() {
    let gil = pyo3::Python::acquire_gil();
    let py = gil.python();
    let circuit = new_circuit(py);

    let overrotation_type = py.get_type::<PragmaOverrotationWrapper>();
    let _new_overrotation_1 = overrotation_type
        .call1(("RotateY".to_string(), vec![1], 20.0, 30.0))
        .unwrap();
    // TypeError('Cannot convert python object to Operation')
    // circuit.call_method1("add", (_new_overrotation_1,)).unwrap();

    let rotatex_type = py.get_type::<RotateXWrapper>();
    let new_rotatex_0 = rotatex_type.call1((0, 0.0)).unwrap();
    circuit.call_method1("add", (new_rotatex_0,)).unwrap();

    let rotatey_type = py.get_type::<RotateYWrapper>();
    let new_rotatey_0 = rotatey_type.call1((0, 1.0)).unwrap();
    circuit.call_method1("add", (new_rotatey_0,)).unwrap();

    let new_rotatey_1 = rotatey_type.call1((1, 2.0)).unwrap();
    circuit.call_method1("add", (new_rotatey_1,)).unwrap();
    let new_rotatey_1 = rotatey_type.call1((1, 3.0)).unwrap();
    circuit.call_method1("add", (new_rotatey_1,)).unwrap();

    // println!("{}", format!("{:?}", circuit.clone()));

    let circuit_overrotated = circuit
        .call_method0("overrotate")
        .unwrap()
        .cast_as::<PyCell<CircuitWrapper>>()
        .unwrap();

    // println!("{}", format!("{:?}", circuit_overrotated.clone()));

    // actually, the original circuit and the overrotated circuit are supposed to be different.
    // test to be adapted, once PragmaOverrotation operation can be added to the circuit without causing an error
    // assert_ne!(format!("{:?}", circuit.clone()), format!("{:?}", circuit_overrotated.clone()));
    assert_eq!(
        format!("{:?}", circuit.clone()),
        format!("{:?}", circuit_overrotated.clone())
    );
}