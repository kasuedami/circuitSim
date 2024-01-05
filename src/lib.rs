use std::{ops::{BitAnd, BitOr, Not}, fmt::Display, num::NonZeroUsize, collections::VecDeque};

use function::Function;
use serde::{Deserialize, Serialize};

pub mod function;

pub struct Simulator {
    circuit: Circuit,
    changed_values: VecDeque<usize>,
    steps_until_unstable: NonZeroUsize,
}

#[derive(Serialize, Deserialize)]
pub struct Circuit {
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    components: Vec<Component>,
    values: Vec<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    value_index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
    value_index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Component {
    input_value_indices: Vec<usize>,
    output_value_indices: Vec<usize>,
    function: Function,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Value {
    On,
    Off,
}

impl Simulator {
    pub fn new() -> Self {
        Self {
            circuit: Circuit::new(),
            changed_values: VecDeque::new(),
            steps_until_unstable: NonZeroUsize::new(1000).unwrap(),
        }
    }

    pub fn from_circuit(circuit: Circuit) -> Self {
        let all_value_indices = (0..circuit.all_values().len()).collect();

        Self {
            circuit,
            changed_values: all_value_indices,
            steps_until_unstable: NonZeroUsize::new(1000).unwrap(),
        }
    }

    pub fn set_input(&mut self, index: usize, value: Value) {
        self.circuit.set_input(index, value);
        self.changed_values.push_back(index);
    }

    pub fn get_input(&mut self, index: usize) -> Value {
        self.circuit.get_input(index)
    }

    pub fn get_output(&self, index: usize) -> Value {
        self.circuit.get_output(index)
    }

    pub fn add_input(&mut self, initial_value: Value) -> (usize, usize) {
        let (input_index, value_index) = self.circuit.add_input(initial_value);
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
        self.circuit.all_values()[output.value_index]
    }

    pub fn step(&mut self) {
        if let Some(value_to_check) = self.changed_values.pop_front() {
            let components_to_update = self.find_components_by_input(value_to_check);

            for component_index in components_to_update {
                let changed_value_indices = self.circuit.evaluate_component(component_index);

                for index in changed_value_indices {
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
            .filter(|(_, component)| component.input_value_indices.contains(&input_value_index))
            .map(|(i, _)| i)
            .collect()
    }
}

impl Circuit {
    pub fn new() -> Self {
        Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
            components: Vec::new(),
            values: Vec::new(),
        }
    }

    pub fn set_input(&mut self, index: usize, value: Value) {
        let value_index = self.inputs[index].value_index;
        self.values[value_index] = value;
    }

    pub fn get_input(&mut self, index: usize) -> Value {
        let value_index = self.inputs[index].value_index;
        self.values[value_index]
    }

    pub fn get_output(&self, index: usize) -> Value {
        let value_index = self.outputs[index].value_index;
        self.values[value_index]
    }

    pub fn add_input(&mut self, initial_value: Value) -> (usize, usize) {
        self.values.push(initial_value);
        let value_index = self.values.len() - 1;
        self.inputs.push(Input { value_index });
        let input_index = self.inputs.len() - 1;

        (input_index, value_index)
    }

    pub fn add_output(&mut self, value_index: usize) -> usize {
        self.outputs.push(Output { value_index });
        let output_index = self.outputs.len() - 1;

        output_index
    }

    pub fn add_component(&mut self, function: Function, input_value_indices: Vec<usize>) -> (usize, Vec<usize>) {
        let output_value_start_index = self.values.len();
        let output_value_indices: Vec<_> = (output_value_start_index..output_value_start_index + function.output_value_count()).collect();
        output_value_indices.iter().for_each(|_| self.values.push(Value::Off));

        let component = Component::new(function, input_value_indices, output_value_indices.clone());
        self.components.push(component);
        let component_index = self.components.len() - 1;

        (component_index, output_value_indices)
    }

    pub fn evaluate_component(&mut self, component_index: usize) -> Vec<usize> {
        let component = &self.components[component_index];
        let input_values: Vec<_> = component.input_value_indices.iter().map(|&input_index| self.values[input_index]).collect();
        let befor_output_values: Vec<_> = component.output_value_indices.iter().map(|&output_index| self.values[output_index]).collect();

        let after_output_values = component.function.evaluate(&input_values);
        let value_changes = befor_output_values.iter().zip(after_output_values.iter())
            .enumerate()
            .filter(|(_, (before, after))| before != after).map(|(i, (_, after))| (i, after))
            .map(|(component_output_index, value)| (component.output_value_indices[component_output_index], value));


        value_changes.clone().for_each(|(output_index, &value)| self.values[output_index] = value);

        value_changes.map(|(output_index, _)| output_index).collect()
    }

    pub fn all_inputs(&self) -> &[Input] {
        &self.inputs
    }

    pub fn all_outputs(&self) -> &[Output] {
        &self.outputs
    }

    pub fn all_components(&self) -> &[Component] {
        &self.components
    }

    pub fn all_values(&self) -> &[Value] {
        &self.values
    }
}

impl Component {
    pub fn new(function: Function, input_value_indices: Vec<usize>, output_value_indices: Vec<usize>) -> Self {
        Self {
            input_value_indices,
            output_value_indices,
            function
        }
    }
}

impl BitAnd for Value {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::On, Value::On) => Value::On,
            _ => Value::Off
        }
    }
}

impl BitOr for Value {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Off, Value::Off) => Value::Off,
            _ => Value::On,
        }
    }
}

impl Not for Value {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Value::On => Value::Off,
            Value::Off => Value::On,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn input_output_test() {
        let mut sim = Circuit::new();
        let (input, value) = sim.add_input(Value::On);
        let output = sim.add_output(value);

        assert_eq!(sim.get_input(input), Value::On);
        assert_eq!(sim.get_output(output), Value::On);

        sim.set_input(input, Value::Off);

        assert_eq!(sim.get_input(input), Value::Off);
        assert_eq!(sim.get_output(output), Value::Off);
    }

    #[test]
    fn and_test() {
        let mut sim = Circuit::new();
        let (input0, value0) = sim.add_input(Value::On);
        let (input1, value1) = sim.add_input(Value::On);

        let (component, values) = sim.add_component(Function::And, vec![value0, value1]);

        let output = sim.add_output(values[0]);

        assert_eq!(sim.get_output(output), Value::Off);

        assert_eq!(sim.evaluate_component(component).len(), 1);
        assert_eq!(sim.get_output(output), Value::On);

        sim.set_input(input0, Value::Off);
        assert_eq!(sim.evaluate_component(component).len(), 1);
        assert_eq!(sim.get_output(output), Value::Off);

        sim.set_input(input1, Value::Off);
        assert_eq!(sim.evaluate_component(component).len(), 0);
        assert_eq!(sim.get_output(output), Value::Off);

        sim.set_input(input0, Value::On);
        assert_eq!(sim.evaluate_component(component).len(), 0);
        assert_eq!(sim.get_output(output), Value::Off);
    }

    #[test]
    fn or_test() {
        let mut sim = Circuit::new();
        let (input0, value0) = sim.add_input(Value::On);
        let (input1, value1) = sim.add_input(Value::On);

        let (component, values) = sim.add_component(Function::Or, vec![value0, value1]);

        let output = sim.add_output(values[0]);

        assert_eq!(sim.get_output(output), Value::Off);

        assert_eq!(sim.evaluate_component(component).len(), 1);
        assert_eq!(sim.get_output(output), Value::On);

        sim.set_input(input0, Value::Off);
        assert_eq!(sim.evaluate_component(component).len(), 0);
        assert_eq!(sim.get_output(output), Value::On);

        sim.set_input(input1, Value::Off);
        assert_eq!(sim.evaluate_component(component).len(), 1);
        assert_eq!(sim.get_output(output), Value::Off);

        sim.set_input(input0, Value::On);
        assert_eq!(sim.evaluate_component(component).len(), 1);
        assert_eq!(sim.get_output(output), Value::On);
    }

    #[test]
    fn not_test() {
        let mut sim = Circuit::new();
        let (input, value) = sim.add_input(Value::On);

        let (component, values) = sim.add_component(Function::Not, vec![value]);

        let output = sim.add_output(values[0]);

        assert_eq!(sim.get_output(output), Value::Off);

        assert_eq!(sim.evaluate_component(component).len(), 0);
        assert_eq!(sim.get_output(output), Value::Off);

        sim.set_input(input, Value::Off);

        assert_eq!(sim.evaluate_component(component).len(), 1);
        assert_eq!(sim.get_output(output), Value::On);
    }
}