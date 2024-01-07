use std::{collections::VecDeque, num::NonZeroUsize};

use crate::{Value, Circuit, function::Function, element::Output};

pub struct Simulator {
    circuit: Circuit,
    values: Vec<Value>,
    changed_values: VecDeque<usize>,
    steps_until_unstable: NonZeroUsize,
}


impl Simulator {
    pub fn new(circuit: Circuit) -> Self {
        let all_value_indices: VecDeque<usize> = (0..circuit.value_list_len()).collect();

        Self {
            circuit: circuit,
            values: vec![Value::Off; all_value_indices.len()],
            changed_values: all_value_indices,
            steps_until_unstable: NonZeroUsize::new(1000).unwrap(),
        }
    }

    pub fn set_input(&mut self, input_index: usize, value: Value) {
        let value_index = self.circuit.input(input_index).value_index();

        if self.values[value_index] != value {
            self.values[value_index] = value;
            self.changed_values.push_back(value_index);
        }
    }

    pub fn get_input_value(&mut self, input_index: usize) -> Value {
        let value_index = self.circuit.input(input_index).value_index();
        self.values[value_index]
    }

    pub fn get_output_value(&self, output_index: usize) -> Value {
        let value_index = self.circuit.output(output_index).value_index();
        self.values[value_index]
    }

    pub fn add_input(&mut self) -> (usize, usize) {
        let (input_index, value_index) = self.circuit.add_input();
        self.changed_values.push_back(value_index);

        (input_index, value_index)
    }

    pub fn add_output(&mut self, value_index: usize) -> usize {
        self.circuit.add_output(value_index)
    }

    pub fn add_component(&mut self, function: Function, input_value_indices: Vec<usize>) -> (usize, Vec<usize>) {
        self.circuit.add_component(function, input_value_indices)
    }

    pub fn circuit(&self) -> &Circuit {
        &self.circuit
    }

    pub fn value_for_output(&self, output: &Output) -> Value {
        self.values[output.value_index()]
    }

    pub fn values(&self) -> &[Value] {
        &self.values
    }

    pub fn value_for_index(&self, value: usize) -> Value {
        self.values[value]
    }

    pub fn step(&mut self) {
        if let Some(value_to_check) = self.changed_values.pop_front() {
            let components_to_update = self.find_components_by_input(value_to_check);

            for component_index in components_to_update {
                let component = self.circuit.component(component_index);
                let input_values: Vec<Value> = component.input_value_indices().iter()
                    .map(|&value_index| self.values[value_index])
                    .collect();
                let old_output_values: Vec<Value> = component.output_value_indices().iter()
                    .map(|&value_index| self.values[value_index])
                    .collect();

                let owned_values = if component.function().output_value_count() != 0 {
                    component.owned_value_indices().iter().map(|&value_index| self.values[value_index]).collect()
                } else {
                    vec![]
                };

                let (new_output_values, new_owned_values) = component.function().evaluate(&input_values, &owned_values);

                for i in 0..component.owned_value_indices().len() {
                    let value_index = component.owned_value_indices()[i];
                    self.values[value_index] = new_owned_values[i];
                }

                let value_changes = old_output_values.iter().zip(new_output_values.iter())
                    .enumerate()
                    .filter(|(_, (before, after))| before != after).map(|(i, (_, after))| (i, after))
                    .map(|(component_output_index, value)| (component.output_value_indices()[component_output_index], value));

                value_changes.clone().for_each(|(output_index, &value)| self.values[output_index] = value);

                for index in value_changes.map(|(output_index, _)| output_index) {
                    if self.changed_values.contains(&index) {
                        self.changed_values.push_back(index);
                    }
                }
            }
        }
    }

    pub fn simulate(&mut self) -> bool {
        let mut step_count: usize = 0;

        while !self.changed_values.is_empty() {
            step_count += 1;

            if step_count > self.steps_until_unstable.into() {
                return false;
            }

            self.step();
        }

        true
    }

    fn find_components_by_input(&mut self, input_value_index: usize) -> Vec<usize> {
        self.circuit.all_components().iter()
            .enumerate()
            .filter(|(_, component)| component.input_value_indices().contains(&input_value_index))
            .map(|(i, _)| i)
            .collect()
    }
}
