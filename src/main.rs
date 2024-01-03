use std::process::exit;

use inquire::{Select, MultiSelect};
use simulator::{Circuit, Function, Value};

const INPUT: &str = "Input";
const OUTPUT: &str = "Output";
const COMPONENT: &str = "Component";
const VALUE: &str = "Value";

const ALL: &str = "All";
const BY_INDEX: &str = "By index";
const CANCEL: &str = "Cancel";

fn main() {
    let mut circuit = initialize();
    let mut running = true;

    while running {
        running = menu(&mut circuit);
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

fn menu(circuit: &mut Circuit) -> bool {

    let options = vec![
        "Add",
        "Interact",
        "Inspect",
        "Exit",
    ];

    let answer = Select::new("What do you want to do?", options).prompt();

    let choice = match answer {
        Ok(choice) => choice,
        Err(_) => simple_error_exiting(),
    };

    match choice {
        "Add" => add(circuit),
        "Interact" => interact(circuit),
        "Inspect" => inspect(circuit),
        "Exit" => {
            println!("Exiting...");
            return false;
        },
        _ => simple_error_exiting(),
    }

    true
}

fn add(circuit: &mut Circuit) {

    let element_options = &[
        INPUT,
        OUTPUT,
        COMPONENT,
        CANCEL,
    ];

    let element_answer = Select::new("Which element should be added?", element_options.to_vec()).prompt();

    if let Ok(choice) = element_answer {
        
        match choice {
            INPUT => add_input(circuit),
            OUTPUT => add_output(circuit),
            COMPONENT => add_component(circuit),
            _ => (),
        }

    } else {
        simple_error();
    }
}

fn add_input(circuit: &mut Circuit) {

    let options = &[Value::On, Value::Off];

    let answer = Select::new("What should be the initial value of the new input?", options.to_vec()).prompt();

    match answer {
        Ok(choice) => {
            let (input_index, value_index) = circuit.add_input(choice);
            println!("New input with index {input_index} and initial value {choice} at value index {value_index} has been added.");
        },
        Err(_) => simple_error(),
    }
}

fn add_output(circuit: &mut Circuit) {

    if circuit.all_values().is_empty() {
        println!("The circuit has no values. Without a value no output can be added.");
        return;
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
}

fn add_component(circuit: &mut Circuit) {

    let functions = &[
        Function::And,
        Function::Or,
        Function::Not,
    ];

    let applicable_functions: Vec<_> = functions.iter().filter(|function| function.input_value_count() <= circuit.all_values().len()).collect();

    if applicable_functions.is_empty() {
        println!("There are no components that can be created because there are to few values that could be used as inputs.");
        return;
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
}

fn interact(circuit: &mut Circuit) {

    let interaction_options = &[
        "Set input",
        "Evaluate component",
    ];

    let interaction_answer = Select::new("Which interaction do you want to do?", interaction_options.to_vec()).prompt();

    if let Ok(interaction_choice) = interaction_answer {
        match interaction_choice {
            "Set input" => set_input(circuit),
            "Evaluate component" => evaluate_component(circuit),
            _ => simple_error()
        }
    }

}

fn set_input(circuit: &mut Circuit) {
    let input_index_options = (0..circuit.all_inputs().len()).collect();
    let input_index_answer = Select::new("Which input value should be set?", input_index_options).prompt();

    if let Ok(input_index_choice) = input_index_answer {

        let value_options = &[
            Value::On,
            Value::Off,
        ];

        let new_value_answer = Select::new("What value should the input be set to?", value_options.to_vec()).prompt();

        if let Ok(new_value_choice) = new_value_answer {
            circuit.set_input(input_index_choice, new_value_choice);
            println!("The value of input {input_index_choice} has been set to {new_value_choice}.");
        }

    } else {
        simple_error();
    }
}

fn evaluate_component(circuit: &mut Circuit) {

    if circuit.all_components().is_empty() {
        println!("There are no components in the circuit.");
        return;
    }

    let component_indices: Vec<_> = (0..circuit.all_components().len()).collect();
    let component_index_answer = Select::new("Which component should be evaluated?", component_indices).prompt();

    match component_index_answer {
        Ok(component_index) => {
            let changed_value_indices = circuit.evaluate_component(component_index);
            println!("Component {component_index} has been evaluated. Values at indices {changed_value_indices:?} have changed.");
        },
        Err(_) => simple_error(),
    }
}

fn inspect(circuit: &mut Circuit) {

    let inspect_options = &[
        INPUT,
        OUTPUT,
        COMPONENT,
        VALUE,
        CANCEL,
    ];

    let inspect_answer = Select::new("Which element should be inspected?", inspect_options.to_vec()).prompt();

    if let Ok(inspect_choice) = inspect_answer {

        let select_options = &[
            ALL,
            BY_INDEX,
            CANCEL,
        ];

        let select_answer = Select::new("Do you want to inspect all or by index?", select_options.to_vec()).prompt();

        if let Ok(select_choice) = select_answer {
            
            match select_choice {
                ALL => {
                    match inspect_choice {
                        INPUT => {
                            println!("Inspecting all inputs:");

                            circuit.all_inputs().iter().enumerate().for_each(|(i, input)| {
                                println!("Index: {i}\n{input:?}");
                            });
                        },
                        OUTPUT => {
                            println!("Inspecting all outputs:");

                            circuit.all_outputs().iter().enumerate().for_each(|(i, input)| {
                                println!("Index: {i}\n{input:?}");
                            });
                        },
                        COMPONENT => {
                            println!("Inspecting all components:");

                            circuit.all_components().iter().enumerate().for_each(|(i, input)| {
                                println!("Index: {i}\n{input:?}");
                            });
                        },
                        VALUE => {
                            println!("Inspecting all values:");

                            circuit.all_values().iter().enumerate().for_each(|(i, input)| {
                                println!("Index: {i}\n{input:?}");
                            });
                        },
                        _ => {
                            simple_error();
                            return;
                        },
                    }
                },
                BY_INDEX => {
                    let choosable_indices: Vec<_> = match inspect_choice {
                        INPUT => (0..circuit.all_inputs().len()).collect(),
                        OUTPUT => (0..circuit.all_outputs().len()).collect(),
                        COMPONENT => (0..circuit.all_components().len()).collect(),
                        VALUE => (0..circuit.all_values().len()).collect(),
                        _ => {
                            simple_error();
                            return;
                        },
                    };

                    let index_answer = Select::new("Which index should be shown?", choosable_indices).prompt();

                    if let Ok(index_choice) = index_answer {
                        match inspect_choice {
                            INPUT => {
                                let choosen_input = &circuit.all_inputs()[index_choice];
                                println!("Inspecting input at index {index_choice}:\n{choosen_input:?}");
                            },
                            OUTPUT => {
                                let choosen_input = &circuit.all_outputs()[index_choice];
                                println!("Inspecting output at index {index_choice}:\n{choosen_input:?}");
                            },
                            COMPONENT => {
                                let choosen_input = &circuit.all_components()[index_choice];
                                println!("Inspecting component at index {index_choice}:\n{choosen_input:?}");
                            },
                            VALUE => {
                                let choosen_input = &circuit.all_values()[index_choice];
                                println!("Inspecting value at index {index_choice}:\n{choosen_input:?}");
                            },
                            _ => {
                                simple_error();
                                return;
                            },
                        }
                    } else {
                        simple_error();
                    }
                },
                _ => simple_error()
            }

        } else {
            simple_error();
            return;
        }

    } else {
        simple_error()
    }
}

fn simple_error() {
    println!("There was an error!");
}

fn simple_error_exiting() -> ! {
    simple_error();
    exit(1)
}