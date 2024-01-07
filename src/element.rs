use serde::{Serialize, Deserialize};

use crate::function::Function;


#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Input {
    value_index: usize,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Output {
    value_index: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Component {
    input_value_indices: Vec<usize>,
    output_value_indices: Vec<usize>,
    owned_value_indices: Vec<usize>,
    function: Function,
}

impl Input {
    pub(super) fn new(value_index: usize) -> Self {
        Input { value_index }
    }

    pub fn value_index(&self) -> usize {
        self.value_index
    }
}

impl Output {
    pub(super) fn new(value_index: usize) -> Self {
        Output { value_index }
    }

    pub fn value_index(&self) -> usize {
        self.value_index
    }
}

impl Component {
    pub fn new(function: Function, input_value_indices: Vec<usize>, output_value_indices: Vec<usize>, owned_value_indices: Vec<usize>) -> Self {
        Self {
            input_value_indices,
            output_value_indices,
            owned_value_indices,
            function,
        }
    }

    pub fn input_value_indices(&self) -> &[usize] {
        &self.input_value_indices
    }

    pub fn output_value_indices(&self) -> &[usize] {
        &self.output_value_indices
    }

    pub fn owned_value_indices(&self) -> &[usize] {
        &self.owned_value_indices
    }

    pub fn function(&self) -> &Function {
        &self.function
    }
}