use std::fmt::Display;

use serde::{Serialize, Deserialize};

use crate::{Value, Circuit, Simulator};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Function {
    And,
    Or,
    Not,
    Nand,
    Nor,
    Circuit(Circuit),
}

impl Function {
    pub fn evaluate(&self, input_values: &[Value]) -> Vec<Value> {
        match self {
            Function::And => {
                let value = input_values.iter().fold(Value::On, |acc, &x| acc & x);
                vec![value]
            },
            Function::Or => {
                let value = input_values.iter().fold(Value::Off, |acc, &x| acc | x);
                vec![value]
            },
            Function::Not => vec![!input_values[0]],
            Function::Nand => {
                let value = !input_values.iter().fold(Value::On, |acc, &x| acc & x);
                vec![value]
            },
            Function::Nor => {
                let value = !input_values.iter().fold(Value::Off, |acc, &x| acc | x);
                vec![value]
            },
            Function::Circuit(circuit) => {
                // TODO: rework this better, at least do some error handling if simulate() -> false

                let mut helper = circuit.clone();

                for i in 0..input_values.len() {
                    helper.set_input(i, input_values[i]);
                }

                let mut simulator = Simulator::from_circuit(helper);
                simulator.simulate();

                simulator.circuit().all_outputs().iter()
                    .map(|ouput| simulator.circuit().all_values()[ouput.value_index])
                    .collect()
            }
        }
    }

    pub fn output_value_count(&self) -> usize {
        match self {
            Function::And => 1,
            Function::Or => 1,
            Function::Not => 1,
            Function::Nand => 1,
            Function::Nor => 1,
            Function::Circuit(circuit) => circuit.all_outputs().len(),
        }
    }

    pub fn input_value_count(&self) -> usize {
        match self {
            Function::And => 2,
            Function::Or => 2,
            Function::Not => 1,
            Function::Nand => 2,
            Function::Nor => 2,
            Function::Circuit(circuit) => circuit.all_inputs().len(),
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn and() {
        let and = Function::And;

        // cases where result should be Value::On
        assert_eq!(and.evaluate(&[Value::On,  Value::On]),  vec![Value::On]);

        // cases where result should be Value::Off
        assert_eq!(and.evaluate(&[Value::On,  Value::Off]), vec![Value::Off]);
        assert_eq!(and.evaluate(&[Value::Off, Value::On]),  vec![Value::Off]);
        assert_eq!(and.evaluate(&[Value::Off, Value::Off]), vec![Value::Off]);
    }

    #[test]
    fn or() {
        let or = Function::Or;

        // cases where result should be Value::On
        assert_eq!(or.evaluate(&[Value::On,  Value::On]),  vec![Value::On]);
        assert_eq!(or.evaluate(&[Value::On,  Value::Off]), vec![Value::On]);
        assert_eq!(or.evaluate(&[Value::Off, Value::On]),  vec![Value::On]);

        // cases where result should be Value::Off
        assert_eq!(or.evaluate(&[Value::Off, Value::Off]), vec![Value::Off]);
    }

    #[test]
    fn not() {
        let not = Function::Not;

        // cases where result should be Value::On
        assert_eq!(not.evaluate(&[Value::Off]), vec![Value::On]);

        // cases where result should be Value::Off
        assert_eq!(not.evaluate(&[Value::On]),  vec![Value::Off]);
    }

    #[test]
    fn nand() {
        let nand = Function::Nand;

        // cases where result should be Value::On
        assert_eq!(nand.evaluate(&[Value::On,  Value::Off]), vec![Value::On]);
        assert_eq!(nand.evaluate(&[Value::Off, Value::On]),  vec![Value::On]);
        assert_eq!(nand.evaluate(&[Value::Off, Value::Off]), vec![Value::On]);

        // cases where result should be Value::Off
        assert_eq!(nand.evaluate(&[Value::On,  Value::On]),  vec![Value::Off]);
    }

    #[test]
    fn nor() {
        let nor = Function::Nor;

        // cases where result should be Value::On
        assert_eq!(nor.evaluate(&[Value::Off, Value::Off]), vec![Value::On]);

        // cases where result should be Value::Off
        assert_eq!(nor.evaluate(&[Value::On,  Value::On]),  vec![Value::Off]);
        assert_eq!(nor.evaluate(&[Value::On,  Value::Off]), vec![Value::Off]);
        assert_eq!(nor.evaluate(&[Value::Off, Value::On]),  vec![Value::Off]);
    }

    #[test]
    fn circuit() {
        let circuit = Function::Circuit(generate_and_circuit());

        // cases where result should be Value::On
        assert_eq!(circuit.evaluate(&[Value::On, Value::On]), vec![Value::On]);

        // cases where result should be Value::Off
        assert_eq!(circuit.evaluate(&[Value::On,  Value::Off]), vec![Value::Off]);
        assert_eq!(circuit.evaluate(&[Value::Off, Value::On]),  vec![Value::Off]);
        assert_eq!(circuit.evaluate(&[Value::Off, Value::Off]), vec![Value::Off]);
    }

    fn generate_and_circuit() -> Circuit {
        let mut circuit = Circuit::new();
        let (_, value0_index) = circuit.add_input(Value::On);
        let (_, value1_index) = circuit.add_input(Value::On);
        let (_, value2_index) = circuit.add_component(Function::And, vec![value0_index, value1_index]);
        let _ = circuit.add_output(value2_index[0]);

        circuit
    }
}