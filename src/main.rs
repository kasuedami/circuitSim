// TODO: remove this
#![allow(dead_code)]

use std::ops::{BitAnd, BitOr, Not};

fn main() {
    println!("Hello, world!");
}

struct Circuit {
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    components: Vec<Component>,
    values: Vec<Value>,
}

struct Input {
    value_index: usize,
}

struct Output {
    value_index: usize,
}

struct Component {
    input_value_indices: Vec<usize>,
    output_value_indices: Vec<usize>,
    function: Function,
}

#[derive(Clone, Copy)]
enum Function {
    And,
    Or,
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Value {
    On,
    Off,
}

enum Element {
    Input(Input),
    Output(Output),
    Component(Component),
}

enum ValueInputIndex {
    Output(usize),
    Component(usize, usize),
}

enum ValueOutputIndex {
    Input(usize),
    Component(usize, usize),
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

        let after_output_values = component.function.evaluate(input_values);
        let value_changes = befor_output_values.iter().zip(after_output_values.iter())
            .enumerate()
            .filter(|(_, (before, after))| before != after).map(|(i, (_, after))| (i, after))
            .map(|(component_output_index, value)| (component.output_value_indices[component_output_index], value));
        

        value_changes.clone().for_each(|(output_index, &value)| self.values[output_index] = value);
        
        value_changes.map(|(output_index, _)| output_index).collect()
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

impl Function {
    pub fn evaluate(&self, input_values: Vec<Value>) -> Vec<Value> {
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
        }
    }

    pub fn output_value_count(&self) -> usize {
        match self {
            Function::And => 1,
            Function::Or => 1,
            Function::Not => 1,
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