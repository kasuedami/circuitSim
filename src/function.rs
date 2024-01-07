use std::fmt::Display;

use serde::{Serialize, Deserialize};

use crate::{Value, Circuit, simulator::Simulator};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Function {
    And,
    Or,
    Not,
    Nand,
    Nor,
    Circuit(Circuit),
    FlipFlopRS,
    FlipFlopJK,
    FlipFlopD,
    FlipFlopT,
}

impl Function {
    pub fn evaluate(&self, input_values: &[Value], owned_values: &[Value]) -> (Vec<Value>, Vec<Value>) {
        match self {
            Function::And => {
                let value = input_values.iter().fold(Value::On, |acc, &x| acc & x);
                (vec![value], vec![])
            },
            Function::Or => {
                let value = input_values.iter().fold(Value::Off, |acc, &x| acc | x);
                (vec![value], vec![])
            },
            Function::Not => (vec![!input_values[0]], vec![]),
            Function::Nand => {
                let value = !input_values.iter().fold(Value::On, |acc, &x| acc & x);
                (vec![value], vec![])
            },
            Function::Nor => {
                let value = !input_values.iter().fold(Value::Off, |acc, &x| acc | x);
                (vec![value], vec![])
            },
            Function::Circuit(circuit) => {
                let mut simulator = Simulator::new(circuit.clone());

                for i in 0..input_values.len() {
                    simulator.set_input(i, input_values[i]);
                }

                simulator.simulate();

                let values = circuit.all_outputs().iter()
                    .map(|output| simulator.value_for_index(output.value_index()))
                    .collect();

                (values, vec![])
            },
            Function::FlipFlopRS => {
                match (input_values[0], input_values[1]) {
                    (Value::On, Value::On) => (vec![Value::Off, Value::Off], owned_values.to_vec()),
                    (Value::Off, Value::Off) => (vec![owned_values[0], !owned_values[0]], owned_values.to_vec()),
                    (set, _) => {
                        (vec![set, !set], vec![set])
                    }
                }
            },
            Function::FlipFlopJK => {
                if is_positiv_transient(owned_values[1], input_values[2]) {
                    let value = match (input_values[0], input_values[1]) {
                        (Value::On, Value::On) => !owned_values[0],
                        (Value::On, Value::Off) => Value::On,
                        (Value::Off, Value::On) => Value::Off,
                        (Value::Off, Value::Off) => owned_values[0],
                    };

                    (vec![value, !value], vec![value, input_values[2]])
                } else {
                    (vec![owned_values[0], !owned_values[1]], vec![owned_values[0], input_values[2]])
                }
            },
            Function::FlipFlopD => {
                if is_positiv_transient(owned_values[1], input_values[1]) {
                    (vec![input_values[0], !input_values[0]], vec![input_values[0], input_values[1]])
                } else {
                    (vec![owned_values[0], !owned_values[1]], vec![owned_values[0], input_values[1]])
                }
            },
            Function::FlipFlopT => {
                if is_positiv_transient(owned_values[1], input_values[1]) && input_values[0] == Value::On {
                    (vec![!owned_values[0], owned_values[0]], vec![!owned_values[0], input_values[1]])
                } else {
                    (vec![owned_values[0], !owned_values[1]], vec![owned_values[0], input_values[1]])
                }
            },
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
            Function::FlipFlopRS => 2,
            Function::FlipFlopJK => 3,
            Function::FlipFlopD => 2,
            Function::FlipFlopT => 2,
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
            Function::FlipFlopRS => 2,
            Function::FlipFlopJK => 2,
            Function::FlipFlopD => 2,
            Function::FlipFlopT => 2,
        }
    }

    pub fn owned_value_count(&self) -> usize {
        match self {
            Function::And => 0,
            Function::Or => 0,
            Function::Not => 0,
            Function::Nand => 0,
            Function::Nor => 0,
            Function::Circuit(_) => 0,
            Function::FlipFlopRS => 1,
            Function::FlipFlopJK => 2,
            Function::FlipFlopD => 2,
            Function::FlipFlopT => 2,
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = format!("{self:?}").chars().take_while(|&ch| ch != '(').collect::<String>();
        write!(f, "{name}")
    }
}

fn is_positiv_transient(old_value: Value, new_value: Value) -> bool {
    old_value != new_value && new_value == Value::On
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn and() {
        let and = Function::And;

        // cases where result should be Value::On
        assert_eq!(and.evaluate(&[Value::On,  Value::On], &[]),  (vec![Value::On], vec![]));

        // cases where result should be Value::Off
        assert_eq!(and.evaluate(&[Value::On,  Value::Off], &[]), (vec![Value::Off], vec![]));
        assert_eq!(and.evaluate(&[Value::Off, Value::On],  &[]), (vec![Value::Off], vec![]));
        assert_eq!(and.evaluate(&[Value::Off, Value::Off], &[]), (vec![Value::Off], vec![]));
    }

    #[test]
    fn or() {
        let or = Function::Or;

        // cases where result should be Value::On
        assert_eq!(or.evaluate(&[Value::On,  Value::On],  &[]), (vec![Value::On], vec![]));
        assert_eq!(or.evaluate(&[Value::On,  Value::Off], &[]), (vec![Value::On], vec![]));
        assert_eq!(or.evaluate(&[Value::Off, Value::On],  &[]), (vec![Value::On], vec![]));

        // cases where result should be Value::Off
        assert_eq!(or.evaluate(&[Value::Off, Value::Off], &[]), (vec![Value::Off], vec![]));
    }

    #[test]
    fn not() {
        let not = Function::Not;

        // cases where result should be Value::On
        assert_eq!(not.evaluate(&[Value::Off], &[]), (vec![Value::On], vec![]));

        // cases where result should be Value::Off
        assert_eq!(not.evaluate(&[Value::On],  &[]), (vec![Value::Off], vec![]));
    }

    #[test]
    fn nand() {
        let nand = Function::Nand;

        // cases where result should be Value::On
        assert_eq!(nand.evaluate(&[Value::On,  Value::Off], &[]), (vec![Value::On], vec![]));
        assert_eq!(nand.evaluate(&[Value::Off, Value::On],  &[]), (vec![Value::On], vec![]));
        assert_eq!(nand.evaluate(&[Value::Off, Value::Off], &[]), (vec![Value::On], vec![]));

        // cases where result should be Value::Off
        assert_eq!(nand.evaluate(&[Value::On,  Value::On],  &[]), (vec![Value::Off], vec![]));
    }

    #[test]
    fn nor() {
        let nor = Function::Nor;

        // cases where result should be Value::On
        assert_eq!(nor.evaluate(&[Value::Off, Value::Off], &[]), (vec![Value::On], vec![]));

        // cases where result should be Value::Off
        assert_eq!(nor.evaluate(&[Value::On,  Value::On],  &[]), (vec![Value::Off], vec![]));
        assert_eq!(nor.evaluate(&[Value::On,  Value::Off], &[]), (vec![Value::Off], vec![]));
        assert_eq!(nor.evaluate(&[Value::Off, Value::On],  &[]), (vec![Value::Off], vec![]));
    }

    #[test]
    fn circuit() {
        let circuit = Function::Circuit(util::generate_and_circuit());

        // cases where result should be Value::On
        assert_eq!(circuit.evaluate(&[Value::On,  Value::On],  &[]), (vec![Value::On], vec![]));

        // cases where result should be Value::Off
        assert_eq!(circuit.evaluate(&[Value::On,  Value::Off], &[]), (vec![Value::Off], vec![]));
        assert_eq!(circuit.evaluate(&[Value::Off, Value::On],  &[]), (vec![Value::Off], vec![]));
        assert_eq!(circuit.evaluate(&[Value::Off, Value::Off], &[]), (vec![Value::Off], vec![]));
    }

    #[test]
    fn flip_flop_rs() {
        let rs = Function::FlipFlopRS;

        // TODO: rewrite this
    }

    #[test]
    fn flip_flop_jk() {
        let jk = Function::FlipFlopJK;

        // TODO: rewrite this
    }

    #[test]
    fn flip_flop_d() {
        let d = Function::FlipFlopD;

        // TODO: rewrite this
    }

    #[test]
    fn flip_flop_t() {
        let t = Function::FlipFlopT;

        // TODO: rewrite this
    }

    mod util {
        use super::super::*;

        pub(super) fn generate_and_circuit() -> Circuit {
            let mut circuit = Circuit::new();
            let (_, value0_index) = circuit.add_input();
            let (_, value1_index) = circuit.add_input();
            let (_, value2_index) = circuit.add_component(Function::And, vec![value0_index, value1_index]);
            let _ = circuit.add_output(value2_index[0]);

            circuit
        }
    }
}