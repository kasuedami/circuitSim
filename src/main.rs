use simulator::{Circuit, Function, Value};

fn main() {
    let mut sim = Circuit::new();
    let (_, value0) = sim.add_input(Value::On);
    let (_, value1) = sim.add_input(Value::On);

    let (_, values) = sim.add_component(Function::Or, vec![value0, value1]);

    let _ = sim.add_output(values[0]);
}