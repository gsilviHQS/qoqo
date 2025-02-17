use roqoqo::{Circuit, operations as ops, CalculatorFloat};
use roqoqo::prelude::EvaluatingBackend;
use roqoqo_quest::Backend;
use core::f64::consts::PI as Pi;
// use qoqo_calculator::CalculatorFloat;

fn main(){

  fn prep_psi (angle_thet: CalculatorFloat, angle_phi: CalculatorFloat) -> Circuit
  {
    let mut circuit = Circuit::new();
    circuit += ops::RotateY::new(0, angle_thet);
    circuit += ops::RotateZ::new(0, angle_phi);
    return circuit
}


let init_circuit = prep_psi(CalculatorFloat::Float(Pi), CalculatorFloat::Float(0f64));


// Preparing an entangled resource state


let mut entangling_circ = Circuit::new();
entangling_circ += ops::Hadamard::new(1);
entangling_circ += ops::CNOT::new(1, 2);

// Encoding the state to be sent in the entangled resource state

let mut encoding_circ = Circuit::new();
encoding_circ += ops::CNOT::new(0, 1);
encoding_circ += ops::Hadamard::new(0);

// State transfer part 1: Measurement

let mut meas_circ = Circuit::new();
meas_circ += ops::DefinitionBit::new("M1M2".to_string(), 2, true);
meas_circ += ops::MeasureQubit::new(0, "M1M2".to_string(), 0);
meas_circ += ops::MeasureQubit::new(1, "M1M2".to_string(), 1);

// Defining the circuit for a conditional operation

let mut conditional_z = Circuit::new();
conditional_z += ops::PauliZ::new(2);

let mut conditional_x = Circuit::new();
conditional_x += ops::PauliX::new(2);

// State transfer part 2: conditional operations

let mut conditional_circ = Circuit::new();
conditional_circ += ops::PragmaConditional::new("M1M2".to_string(), 1, conditional_x);
conditional_circ += ops::PragmaConditional::new("M1M2".to_string(), 0, conditional_z);

// Putting it all together

let mut verification = Circuit::new();
verification += ops::DefinitionComplex::new("psi".to_string(), 8, true);
verification += ops::PragmaGetStateVector::new("psi".to_string(), Some(Circuit::new()));

let teleportation_circuit = init_circuit + entangling_circ + encoding_circ + meas_circ + conditional_circ + verification;

let backend = Backend::new(3);
let result_of_run = backend.run_circuit(&teleportation_circuit);
let (result_bit_registers, _result_float_registers, result_complex_registers) 
= result_of_run.unwrap();

println!("Result bit registers :{:?}",result_bit_registers["M1M2"]);
println!("Result complex registers :{:?}",result_complex_registers["psi"]);

}