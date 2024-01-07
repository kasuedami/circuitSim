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
    FlipFlopRS(Value),
    FlipFlopJK(FlipFlopJK),
    FlipFlopD(FlipFlopD),
    FlipFlopT(FlipFlopT),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct TransientDetection {
    old_value: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FlipFlopJK {
    clock_input: TransientDetection,
    state: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FlipFlopD {
    clock_input: TransientDetection,
    state: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FlipFlopT {
    clock_input: TransientDetection,
    state: Value,
}

impl Function {
    pub fn evaluate(&mut self, input_values: &[Value]) -> Vec<Value> {
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
                let mut simulator = Simulator::new(circuit.clone());

                for i in 0..input_values.len() {
                    simulator.set_input(i, input_values[i]);
                }

                simulator.simulate();

                circuit.all_outputs().iter()
                    .map(|output| simulator.value_for_index(output.value_index()))
                    .collect()
            },
            Function::FlipFlopRS(state) => {
                match (input_values[0], input_values[1]) {
                    (Value::On, Value::On) => vec![Value::Off, Value::Off],
                    (Value::Off, Value::Off) => vec![*state, !*state],
                    (set, _) => {
                        *state = set;
                        vec![*state, !*state]
                    }
                }
            },
            Function::FlipFlopJK(flip_flop_jk) => {
                if flip_flop_jk.clock_input.is_transient(input_values[2]) && input_values[2] == Value::On {
                    match (input_values[0], input_values[1]) {
                        (Value::On, Value::On) => flip_flop_jk.state = !flip_flop_jk.state,
                        (Value::On, Value::Off) => flip_flop_jk.state = Value::On,
                        (Value::Off, Value::On) => flip_flop_jk.state = Value::Off,
                        (Value::Off, Value::Off) => (),
                    }
                }

                vec![flip_flop_jk.state, !flip_flop_jk.state]
            }
            Function::FlipFlopD(flip_flop_d) => {
                if flip_flop_d.clock_input.is_transient(input_values[1]) && input_values[1] == Value::On {
                    flip_flop_d.state = input_values[0];
                }

                vec![flip_flop_d.state, !flip_flop_d.state]
            },
            Function::FlipFlopT(flip_flop_t) => {
                if flip_flop_t.clock_input.is_transient(input_values[1]) && input_values[1] == Value::On {
                    if input_values[0] == Value::On {
                        flip_flop_t.state = !flip_flop_t.state;
                    }
                }

                vec![flip_flop_t.state, !flip_flop_t.state]
            }
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
            Function::FlipFlopRS(_) => 2,
            Function::FlipFlopJK(_) => 3,
            Function::FlipFlopD(_) => 2,
            Function::FlipFlopT(_) => 2,
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
            Function::FlipFlopRS(_) => 2,
            Function::FlipFlopJK(_) => 2,
            Function::FlipFlopD(_) => 2,
            Function::FlipFlopT(_) => 2,
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = format!("{self:?}").chars().take_while(|&ch| ch != '(').collect::<String>();
        write!(f, "{name}")
    }
}

impl TransientDetection {
    fn is_transient(&mut self, new_value: Value) -> bool {
        if self.old_value != new_value {
            self.old_value = new_value;
            true
        } else {
            false
        }
    }
}

impl FlipFlopJK {
    pub fn new(initial_value: Value) -> Self {
        Self {
            clock_input: TransientDetection {
                old_value: Value::Off,
            },
            state: initial_value,
        }
    }
}

impl FlipFlopD {
    pub fn new(initial_value: Value) -> Self {
        Self {
            clock_input: TransientDetection {
                old_value: Value::Off,
            },
            state: initial_value,
        }
    }
}

impl FlipFlopT {
    pub fn new(initial_value: Value) -> Self {
        Self {
            clock_input: TransientDetection {
                old_value: Value::Off,
            },
            state: initial_value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn and() {
        let mut and = Function::And;

        // cases where result should be Value::On
        assert_eq!(and.evaluate(&[Value::On,  Value::On]),  vec![Value::On]);

        // cases where result should be Value::Off
        assert_eq!(and.evaluate(&[Value::On,  Value::Off]), vec![Value::Off]);
        assert_eq!(and.evaluate(&[Value::Off, Value::On]),  vec![Value::Off]);
        assert_eq!(and.evaluate(&[Value::Off, Value::Off]), vec![Value::Off]);
    }

    #[test]
    fn or() {
        let mut or = Function::Or;

        // cases where result should be Value::On
        assert_eq!(or.evaluate(&[Value::On,  Value::On]),  vec![Value::On]);
        assert_eq!(or.evaluate(&[Value::On,  Value::Off]), vec![Value::On]);
        assert_eq!(or.evaluate(&[Value::Off, Value::On]),  vec![Value::On]);

        // cases where result should be Value::Off
        assert_eq!(or.evaluate(&[Value::Off, Value::Off]), vec![Value::Off]);
    }

    #[test]
    fn not() {
        let mut not = Function::Not;

        // cases where result should be Value::On
        assert_eq!(not.evaluate(&[Value::Off]), vec![Value::On]);

        // cases where result should be Value::Off
        assert_eq!(not.evaluate(&[Value::On]),  vec![Value::Off]);
    }

    #[test]
    fn nand() {
        let mut nand = Function::Nand;

        // cases where result should be Value::On
        assert_eq!(nand.evaluate(&[Value::On,  Value::Off]), vec![Value::On]);
        assert_eq!(nand.evaluate(&[Value::Off, Value::On]),  vec![Value::On]);
        assert_eq!(nand.evaluate(&[Value::Off, Value::Off]), vec![Value::On]);

        // cases where result should be Value::Off
        assert_eq!(nand.evaluate(&[Value::On,  Value::On]),  vec![Value::Off]);
    }

    #[test]
    fn nor() {
        let mut nor = Function::Nor;

        // cases where result should be Value::On
        assert_eq!(nor.evaluate(&[Value::Off, Value::Off]), vec![Value::On]);

        // cases where result should be Value::Off
        assert_eq!(nor.evaluate(&[Value::On,  Value::On]),  vec![Value::Off]);
        assert_eq!(nor.evaluate(&[Value::On,  Value::Off]), vec![Value::Off]);
        assert_eq!(nor.evaluate(&[Value::Off, Value::On]),  vec![Value::Off]);
    }

    #[test]
    fn circuit() {
        let mut circuit = Function::Circuit(util::generate_and_circuit());

        // cases where result should be Value::On
        assert_eq!(circuit.evaluate(&[Value::On, Value::On]), vec![Value::On]);

        // cases where result should be Value::Off
        assert_eq!(circuit.evaluate(&[Value::On,  Value::Off]), vec![Value::Off]);
        assert_eq!(circuit.evaluate(&[Value::Off, Value::On]),  vec![Value::Off]);
        assert_eq!(circuit.evaluate(&[Value::Off, Value::Off]), vec![Value::Off]);
    }

    #[test]
    fn transient_detection() {
        let mut transient_detection = TransientDetection {
            old_value: Value::Off,
        };

        assert_eq!(transient_detection.is_transient(Value::Off), false);
        assert_eq!(transient_detection.is_transient(Value::On), true);
        assert_eq!(transient_detection.is_transient(Value::On), false);
        assert_eq!(transient_detection.is_transient(Value::Off), true);
    }

    #[test]
    fn flip_flop_rs() {
        let mut rs = Function::FlipFlopRS(Value::On);

        let main_on = &[Value::On, Value::Off];
        let main_off = &[Value::Off, Value::On];

        assert_eq!(rs.evaluate(&[Value::Off, Value::Off]), main_on);
        assert_eq!(rs.evaluate(&[Value::On, Value::Off]), main_on);
        assert_eq!(rs.evaluate(&[Value::On, Value::On]), &[Value::Off, Value::Off]);
        assert_eq!(rs.evaluate(&[Value::Off, Value::Off]), main_on);
        assert_eq!(rs.evaluate(&[Value::Off, Value::On]), main_off);
        assert_eq!(rs.evaluate(&[Value::Off, Value::On]), main_off);
        assert_eq!(rs.evaluate(&[Value::Off, Value::Off]), main_off);

    }

    #[test]
    fn flip_flop_jk() {
        let mut jk = Function::FlipFlopJK(FlipFlopJK::new(Value::On));

        let main_on = &[Value::On, Value::Off];
        let main_off = &[Value::Off, Value::On];

        assert_eq!(jk.evaluate(&[Value::Off, Value::Off, Value::On]), main_on);
        assert_eq!(jk.evaluate(&[Value::On, Value::Off, Value::On]), main_on);
        assert_eq!(jk.evaluate(&[Value::Off, Value::On, Value::On]), main_on);
        assert_eq!(jk.evaluate(&[Value::Off, Value::On, Value::Off]), main_on);

        assert_eq!(jk.evaluate(&[Value::On, Value::On, Value::On]), main_off);
        assert_eq!(jk.evaluate(&[Value::On, Value::On, Value::Off]), main_off);
        assert_eq!(jk.evaluate(&[Value::On, Value::On, Value::On]), main_on);
        assert_eq!(jk.evaluate(&[Value::Off, Value::Off, Value::Off]), main_on);
        assert_eq!(jk.evaluate(&[Value::Off, Value::On, Value::On]), main_off);

        assert_eq!(jk.evaluate(&[Value::On, Value::Off, Value::On]), main_off);
        assert_eq!(jk.evaluate(&[Value::On, Value::On, Value::On]), main_off);
        assert_eq!(jk.evaluate(&[Value::On, Value::On, Value::Off]), main_off);
        assert_eq!(jk.evaluate(&[Value::Off, Value::Off, Value::On]), main_off);
        assert_eq!(jk.evaluate(&[Value::Off, Value::On, Value::On]), main_off);
        assert_eq!(jk.evaluate(&[Value::On, Value::Off, Value::Off]), main_off);

        assert_eq!(jk.evaluate(&[Value::On, Value::Off, Value::On]), main_on);
    }

    #[test]
    fn flip_flop_d() {
        let mut d = Function::FlipFlopD(FlipFlopD::new(Value::On));

        let main_on = &[Value::On, Value::Off];
        let main_off = &[Value::Off, Value::On];

        assert_eq!(d.evaluate(&[Value::Off, Value::Off]), main_on);
        assert_eq!(d.evaluate(&[Value::On, Value::Off]), main_on);
        assert_eq!(d.evaluate(&[Value::On, Value::On]), main_on);
        assert_eq!(d.evaluate(&[Value::Off, Value::On]), main_on);
        assert_eq!(d.evaluate(&[Value::Off, Value::Off]), main_on);
        assert_eq!(d.evaluate(&[Value::Off, Value::On]), main_off);
        assert_eq!(d.evaluate(&[Value::Off, Value::Off]), main_off);
        assert_eq!(d.evaluate(&[Value::On, Value::Off]), main_off);
        assert_eq!(d.evaluate(&[Value::Off, Value::On]), main_off);
    }

    #[test]
    fn flip_flop_t() {
        let mut t = Function::FlipFlopT(FlipFlopT::new(Value::On));

        let main_on = &[Value::On, Value::Off];
        let main_off = &[Value::Off, Value::On];

        assert_eq!(t.evaluate(&[Value::Off, Value::Off]), main_on);
        assert_eq!(t.evaluate(&[Value::Off, Value::On]), main_on);
        assert_eq!(t.evaluate(&[Value::On, Value::Off]), main_on);
        assert_eq!(t.evaluate(&[Value::On, Value::On]), main_off);
        assert_eq!(t.evaluate(&[Value::On, Value::Off]), main_off);
        assert_eq!(t.evaluate(&[Value::Off, Value::On]), main_off);
        assert_eq!(t.evaluate(&[Value::On, Value::Off]), main_off);
        assert_eq!(t.evaluate(&[Value::Off, Value::Off]), main_off);
        assert_eq!(t.evaluate(&[Value::On, Value::On]), main_on);
        assert_eq!(t.evaluate(&[Value::On, Value::Off]), main_on);
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