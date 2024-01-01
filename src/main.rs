// TODO: remove this
#![allow(dead_code)]

fn main() {
    println!("Hello, world!");
}

struct Simulator {
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
    output_value_indces: Vec<usize>,
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

impl Simulator {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic() {
        let mut sim = Simulator::new();
        let (input, value) = sim.add_input(Value::On);
        let output = sim.add_output(value);

        assert_eq!(sim.get_input(input), Value::On);
        assert_eq!(sim.get_output(output), Value::On);

        sim.set_input(input, Value::Off);

        assert_eq!(sim.get_input(input), Value::Off);
        assert_eq!(sim.get_output(output), Value::Off);
    }
}