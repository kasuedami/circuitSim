use std::{ops::{BitAnd, BitOr, Not}, fmt::Display};

use element::{Input, Output, Component};
use function::Function;
use serde::{Deserialize, Serialize};

pub mod function;
pub mod element;
pub mod simulator;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Circuit {
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    components: Vec<Component>,
    value_list_len: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Value {
    On,
    Off,
}

impl Circuit {
    pub fn new() -> Self {
        Self {
            inputs: Vec::new(),
            outputs: Vec::new(),
            components: Vec::new(),
            value_list_len: 0,
        }
    }

    pub fn add_input(&mut self) -> (usize, usize) {
        let value_index = self.value_list_len;
        self.value_list_len += 1;

        self.inputs.push(Input::new(value_index));
        let input_index = self.inputs.len() - 1;

        (input_index, value_index)
    }

    pub fn add_output(&mut self, value_index: usize) -> usize {
        self.outputs.push(Output::new(value_index));
        let output_index = self.outputs.len() - 1;

        output_index
    }

    pub fn add_component(&mut self, function: Function, input_value_indices: Vec<usize>) -> (usize, Vec<usize>) {
        let output_value_start_index = self.value_list_len;
        self.value_list_len += function.output_value_count();
        let output_value_indices: Vec<usize> = (output_value_start_index..self.value_list_len).collect();

        let owned_value_start_index = self.value_list_len;
        self.value_list_len += function.owned_value_count();
        let owned_value_indices: Vec<usize> = (owned_value_start_index..self.value_list_len).collect();

        let component = Component::new(function, input_value_indices, output_value_indices.clone(), owned_value_indices);
        self.components.push(component);
        let component_index = self.components.len() - 1;

        (component_index, output_value_indices)
    }

    pub fn input(&self, input_index: usize) -> &Input {
        &self.inputs[input_index]
    }

    pub fn output(&self, output_index: usize) -> &Output {
        &self.outputs[output_index]
    }

    pub fn component(&self, component_index: usize) -> &Component {
        &self.components[component_index]
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

    pub fn value_list_len(&self) -> usize {
        self.value_list_len
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