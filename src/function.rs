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
                    (vec![owned_values[0], !owned_values[0]], vec![owned_values[0], input_values[2]])
                }
            },
            Function::FlipFlopD => {
                if is_positiv_transient(owned_values[1], input_values[1]) {
                    (vec![input_values[0], !input_values[0]], vec![input_values[0], input_values[1]])
                } else {
                    (vec![owned_values[0], !owned_values[0]], vec![owned_values[0], input_values[1]])
                }
            },
            Function::FlipFlopT => {
                if is_positiv_transient(owned_values[1], input_values[1]) && input_values[0] == Value::On {
                    (vec![!owned_values[0], owned_values[0]], vec![!owned_values[0], input_values[1]])
                } else {
                    (vec![owned_values[0], !owned_values[0]], vec![owned_values[0], input_values[1]])
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
    use itertools::Itertools;

    use self::util::{ClockState, tripple_input, dual_input};

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

        let on_off =  &[Value::On,  Value::Off];
        let off_on =  &[Value::Off, Value::On];
        let on_on =   &[Value::On,  Value::On];
        let off_off = &[Value::Off, Value::Off];

        let on = &[Value::On];
        let off = &[Value::Off];

        let (output_values, owned_values) = rs.evaluate(off_off, off);
        assert_eq!(output_values, off_on);
        assert_eq!(owned_values, off);

        let (output_values, owned_values) = rs.evaluate(off_off, on);
        assert_eq!(output_values, on_off);
        assert_eq!(owned_values, on);

        let (output_values, owned_values) = rs.evaluate(off_on, off);
        assert_eq!(output_values, off_on);
        assert_eq!(owned_values, off);

        let (output_values, owned_values) = rs.evaluate(off_on, on);
        assert_eq!(output_values, off_on);
        assert_eq!(owned_values, off);

        let (output_values, owned_values) = rs.evaluate(on_off, off);
        assert_eq!(output_values, on_off);
        assert_eq!(owned_values, on);

        let (output_values, owned_values) = rs.evaluate(on_off, on);
        assert_eq!(output_values, on_off);
        assert_eq!(owned_values, on);

        let (output_values, owned_values) = rs.evaluate(on_on, off);
        assert_eq!(output_values, off_off);
        assert_eq!(owned_values, off);

        let (output_values, owned_values) = rs.evaluate(on_on, on);
        assert_eq!(output_values, off_off);
        assert_eq!(owned_values, on);
    }

    #[test]
    fn flip_flop_jk() {
        let jk = Function::FlipFlopJK;

        let on_off =  &[Value::On,  Value::Off];
        let off_on =  &[Value::Off, Value::On];
        let on_on =   &[Value::On,  Value::On];

        let base = vec![Value::On, Value::On, Value::On, Value::Off, Value::Off, Value::Off];
        let j_k_state_combinations: Vec<_> = base.iter().permutations(3).unique().collect();

        // in all these cases the output value should not change an thus be the same as the old state
        for j_k_state in j_k_state_combinations {
            let j = *j_k_state[0];
            let k = *j_k_state[1];
            let state = *j_k_state[2];

            let (input_values, owned_values) = tripple_input(j, k, state, ClockState::StayOff);
            let (output_values, owned_values) = jk.evaluate(&input_values, &owned_values);
            assert_eq!(output_values, &[state, !state]);
            assert_eq!(owned_values, &[state, Value::Off]);

            let (input_values, owned_values) = tripple_input(j, k, state, ClockState::StayOn);
            let (output_values, owned_values) = jk.evaluate(&input_values, &owned_values);
            assert_eq!(output_values, &[state, !state]);
            assert_eq!(owned_values, &[state, Value::On]);

            let (input_values, owned_values) = tripple_input(j, k, state, ClockState::TransientToOff);
            let (output_values, owned_values) = jk.evaluate(&input_values, &owned_values);
            assert_eq!(output_values, &[state, !state]);
            assert_eq!(owned_values, &[state, Value::Off]);
        }

        // these are the cases where actual logic is happening
        let (input_values, owned_values) = tripple_input(Value::Off, Value::Off, Value::Off, ClockState::TransientToOn);
        let (output_values, owned_values) = jk.evaluate(&input_values, &owned_values);
        assert_eq!(output_values, off_on);
        assert_eq!(owned_values, off_on);

        let (input_values, owned_values) = tripple_input(Value::On, Value::Off, Value::Off, ClockState::TransientToOn);
        let (output_values, owned_values) = jk.evaluate(&input_values, &owned_values);
        assert_eq!(output_values, on_off);
        assert_eq!(owned_values, on_on);

        let (input_values, owned_values) = tripple_input(Value::Off, Value::On, Value::Off, ClockState::TransientToOn);
        let (output_values, owned_values) = jk.evaluate(&input_values, &owned_values);
        assert_eq!(output_values, off_on);
        assert_eq!(owned_values, off_on);

        let (input_values, owned_values) = tripple_input(Value::Off, Value::Off, Value::On, ClockState::TransientToOn);
        let (output_values, owned_values) = jk.evaluate(&input_values, &owned_values);
        assert_eq!(output_values, on_off);
        assert_eq!(owned_values, on_on);

        let (input_values, owned_values) = tripple_input(Value::On, Value::On, Value::Off, ClockState::TransientToOn);
        let (output_values, owned_values) = jk.evaluate(&input_values, &owned_values);
        assert_eq!(output_values, on_off);
        assert_eq!(owned_values, on_on);

        let (input_values, owned_values) = tripple_input(Value::On, Value::Off, Value::On, ClockState::TransientToOn);
        let (output_values, owned_values) = jk.evaluate(&input_values, &owned_values);
        assert_eq!(output_values, on_off);
        assert_eq!(owned_values, on_on);

        let (input_values, owned_values) = tripple_input(Value::Off, Value::On, Value::On, ClockState::TransientToOn);
        let (output_values, owned_values) = jk.evaluate(&input_values, &owned_values);
        assert_eq!(output_values, off_on);
        assert_eq!(owned_values, off_on);

        let (input_values, owned_values) = tripple_input(Value::On, Value::On, Value::On, ClockState::TransientToOn);
        let (output_values, owned_values) = jk.evaluate(&input_values, &owned_values);
        assert_eq!(output_values, off_on);
        assert_eq!(owned_values, off_on);
    }

    #[test]
    fn flip_flop_d() {
        let d = Function::FlipFlopD;

        let on_off =  &[Value::On,  Value::Off];
        let off_on =  &[Value::Off, Value::On];
        let on_on =   &[Value::On,  Value::On];

        let base = vec![Value::On, Value::On, Value::Off, Value::Off];
        let d_state_combinations: Vec<_> = base.iter().permutations(2).unique().collect();

        // in all these cases the output value should not change an thus be the same as the old state
        for d_state in d_state_combinations {
            let d_input = *d_state[0];
            let state = *d_state[1];

            let (input_values, owned_values) = dual_input(d_input, state, ClockState::StayOff);
            let (output_values, owned_values) = d.evaluate(&input_values, &owned_values);
            assert_eq!(output_values, &[state, !state]);
            assert_eq!(owned_values, &[state, Value::Off]);

            let (input_values, owned_values) = dual_input(d_input, state, ClockState::StayOn);
            let (output_values, owned_values) = d.evaluate(&input_values, &owned_values);
            assert_eq!(output_values, &[state, !state]);
            assert_eq!(owned_values, &[state, Value::On]);

            let (input_values, owned_values) = dual_input(d_input, state, ClockState::TransientToOff);
            let (output_values, owned_values) = d.evaluate(&input_values, &owned_values);
            assert_eq!(output_values, &[state, !state]);
            assert_eq!(owned_values, &[state, Value::Off]);
        }

        // these are the cases where actual logic is happening
        let (input_values, owned_values) = dual_input(Value::Off, Value::Off, ClockState::TransientToOn);
        let (output_values, owned_values) = d.evaluate(&input_values, &owned_values);
        assert_eq!(output_values, off_on);
        assert_eq!(owned_values, off_on);

        let (input_values, owned_values) = dual_input(Value::On, Value::Off, ClockState::TransientToOn);
        let (output_values, owned_values) = d.evaluate(&input_values, &owned_values);
        assert_eq!(output_values, on_off);
        assert_eq!(owned_values, on_on);

        let (input_values, owned_values) = dual_input(Value::Off, Value::On, ClockState::TransientToOn);
        let (output_values, owned_values) = d.evaluate(&input_values, &owned_values);
        assert_eq!(output_values, off_on);
        assert_eq!(owned_values, off_on);

        let (input_values, owned_values) = dual_input(Value::On, Value::On, ClockState::TransientToOn);
        let (output_values, owned_values) = d.evaluate(&input_values, &owned_values);
        assert_eq!(output_values, on_off);
        assert_eq!(owned_values, on_on);
    }

    #[test]
    fn flip_flop_t() {
        let t = Function::FlipFlopT;

        let on_off =  &[Value::On,  Value::Off];
        let off_on =  &[Value::Off, Value::On];
        let on_on =   &[Value::On,  Value::On];

        let base = vec![Value::On, Value::On, Value::Off, Value::Off];
        let d_state_combinations: Vec<_> = base.iter().permutations(2).unique().collect();

        // in all these cases the output value should not change an thus be the same as the old state
        for d_state in d_state_combinations {
            let d_input = *d_state[0];
            let state = *d_state[1];

            let (input_values, owned_values) = dual_input(d_input, state, ClockState::StayOff);
            let (output_values, owned_values) = t.evaluate(&input_values, &owned_values);
            assert_eq!(output_values, &[state, !state]);
            assert_eq!(owned_values, &[state, Value::Off]);

            let (input_values, owned_values) = dual_input(d_input, state, ClockState::StayOn);
            let (output_values, owned_values) = t.evaluate(&input_values, &owned_values);
            assert_eq!(output_values, &[state, !state]);
            assert_eq!(owned_values, &[state, Value::On]);

            let (input_values, owned_values) = dual_input(d_input, state, ClockState::TransientToOff);
            let (output_values, owned_values) = t.evaluate(&input_values, &owned_values);
            assert_eq!(output_values, &[state, !state]);
            assert_eq!(owned_values, &[state, Value::Off]);
        }

        // these are the cases where actual logic is happening
        let (input_values, owned_values) = dual_input(Value::Off, Value::Off, ClockState::TransientToOn);
        let (output_values, owned_values) = t.evaluate(&input_values, &owned_values);
        assert_eq!(output_values, off_on);
        assert_eq!(owned_values, off_on);

        let (input_values, owned_values) = dual_input(Value::On, Value::Off, ClockState::TransientToOn);
        let (output_values, owned_values) = t.evaluate(&input_values, &owned_values);
        assert_eq!(output_values, on_off);
        assert_eq!(owned_values, on_on);

        let (input_values, owned_values) = dual_input(Value::Off, Value::On, ClockState::TransientToOn);
        let (output_values, owned_values) = t.evaluate(&input_values, &owned_values);
        assert_eq!(output_values, on_off);
        assert_eq!(owned_values, on_on);

        let (input_values, owned_values) = dual_input(Value::On, Value::On, ClockState::TransientToOn);
        let (output_values, owned_values) = t.evaluate(&input_values, &owned_values);
        assert_eq!(output_values, off_on);
        assert_eq!(owned_values, off_on);
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

        pub(super) enum ClockState {
            StayOff,
            StayOn,
            TransientToOn,
            TransientToOff,
        }

        pub(super) fn dual_input(i: Value, state: Value, clock: ClockState) -> (Vec<Value>, Vec<Value>) {
            let input_values = vec![i, match clock {
                ClockState::StayOff => Value::Off,
                ClockState::StayOn => Value::On,
                ClockState::TransientToOn => Value::On,
                ClockState::TransientToOff => Value::Off,
            }];

            let owned_value = vec![state, match clock {
                ClockState::StayOff => Value::Off,
                ClockState::StayOn => Value::On,
                ClockState::TransientToOn => Value::Off,
                ClockState::TransientToOff => Value::On,
            }];

            (input_values, owned_value)
        }

        pub(super) fn tripple_input(j: Value, k: Value, state: Value, clock: ClockState) -> (Vec<Value>, Vec<Value>) {
            let input_values = vec![j, k, match clock {
                ClockState::StayOff => Value::Off,
                ClockState::StayOn => Value::On,
                ClockState::TransientToOn => Value::On,
                ClockState::TransientToOff => Value::Off,
            }];

            let owned_value = vec![state, match clock {
                ClockState::StayOff => Value::Off,
                ClockState::StayOn => Value::On,
                ClockState::TransientToOn => Value::Off,
                ClockState::TransientToOff => Value::On,
            }];

            (input_values, owned_value)
        }
    }
}