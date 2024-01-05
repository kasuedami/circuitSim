use std::fmt::Display;

use serde::{Serialize, Deserialize};

use crate::Value;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Function {
    And,
    Or,
    Not,
    Nand,
    Nor,
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
        }
    }

    pub fn input_value_count(&self) -> usize {
        match self {
            Function::And => 2,
            Function::Or => 2,
            Function::Not => 1,
            Function::Nand => 2,
            Function::Nor => 2,
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
}