use std::process::exit;

use inquire::{Select, MultiSelect};
use simulator::{Circuit, Function, Value};

fn main() {
    let mut circuit = initialize();
    let mut running = true;

    while running {
        running = interaction(&mut circuit);
    }
}

fn initialize() -> Circuit {
    let options = vec![
        "Yes",
        "No",
    ];

    let answer = Select::new("Do you want to create a new circuit simulation?", options).prompt();

    match answer {
        Ok(choice) => {
            match choice {
                "Yes" => {
                    println!("Creating new empty circuit simulation!");
                    Circuit::new()
                },
                "No" => {
                    println!("Loading circuit simulations is currently not supported!");
                    exit(0);
                },
                _ => simple_error_exiting(),
            }
        },
        Err(_) => {
            simple_error_exiting();
        }
    }
}

fn interaction(circuit: &mut Circuit) -> bool {

    let options = vec![
        "New input",
        "New output",
        "New component",
        "Show all inputs",
        "Show all outputs",
        "Show all component",
        "Read input",
        "Read output",
        "Read value",
        "Exit",
    ];

    let answer = Select::new("What do you want to do?", options).prompt();

    let choice = match answer {
        Ok(choice) => choice,
        Err(_) => simple_error_exiting(),
    };

    match choice {
        "New input" => add_input(circuit),
        "New output" => add_output(circuit),
        "New component" => add_component(circuit),
        "Show all inputs" => true,
        "Show all outputs" => true,
        "Show all component" => true,
        "Read input" => true,
        "Read output" => true,
        "Read component" => true,
        "Read value" => true,
        "Exit" => {
            println!("Exiting...");
            false
        },
        _ => simple_error_exiting(),
    }
}

fn add_input(circuit: &mut Circuit) -> bool {

    let options = &[Value::On, Value::Off];

    let answer = Select::new("What should be the initial value of the new input?", options.to_vec()).prompt();

    match answer {
        Ok(choice) => {
            let (input_index, value_index) = circuit.add_input(choice);
            println!("New input with index {input_index} and initial value {choice} at value index {value_index} has been added.");
        },
        Err(_) => simple_error(),
    }

    true
}

fn add_output(circuit: &mut Circuit) -> bool {

    if circuit.all_values().is_empty() {
        println!("The circuit has no values. Without a value no output can be added.");
        return true;
    }

    let options: Vec<_> = (0..circuit.all_values().len()).collect();

    let answer = Select::new("Which value should the new output read?", options).prompt();

    match answer {
        Ok(choice) => {
            let output_index = circuit.add_output(choice);
            println!("New output with index {output_index} reading value form {choice} has been added.");
        },
        Err(_) => simple_error(),
    }

    true
}

fn add_component(circuit: &mut Circuit) -> bool {

    let functions = &[
        Function::And,
        Function::Or,
        Function::Not,
    ];

    let applicable_functions: Vec<_> = functions.iter().filter(|function| function.input_value_count() <= circuit.all_values().len()).collect();

    if applicable_functions.is_empty() {
        println!("There are no components that can be created because there are to few values that could be used as inputs.");
        return true;
    }

    let funtion_answer = Select::new("Which function should the new component be using?", applicable_functions).prompt();

    match funtion_answer {
        Ok(&function_choice) => {
            let input_value_indices: Vec<_> = (0..circuit.all_values().len()).collect();

            // TODO: validators for ensuring min/max ammounts of inputs are chosen
            let input_answer = MultiSelect::new("Choose the values to use as inputs for the component:", input_value_indices).prompt();

            match input_answer {
                Ok(input_choice) => {
                    let (component_index, output_indices) = circuit.add_component(function_choice, input_choice.clone());
                    println!("Component with index {component_index} using function {function_choice} on inputs {input_choice:?} with outputs {output_indices:?} has been added.")
                },
                Err(_) => simple_error(),
            }
        },
        Err(_) => simple_error(),
    }

    true
}

fn simple_error() {
    println!("There was an error!");
}

fn simple_error_exiting() -> ! {
    simple_error();
    exit(1)
}