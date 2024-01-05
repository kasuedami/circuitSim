use std::{process::exit, fs, io::Write};

use inquire::{Select, MultiSelect, list_option::ListOption, validator::Validation, Text};
use simulator::{function::Function, Value, Simulator, Circuit};

mod cli_util;

const INPUT: &str = "Input";
const OUTPUT: &str = "Output";
const COMPONENT: &str = "Component";
const VALUE: &str = "Value";

const ALL: &str = "All";
const BY_INDEX: &str = "By index";

fn main() {
    let mut simulator = initialize();
    let mut running = true;

    while running {
        running = menu(&mut simulator);
    }
}

fn initialize() -> Simulator {
    let options = vec![
        "New",
        "Load",
    ];

    let answer = Select::new("Do you want to create a new circuit simulation or load an existing?", options).prompt();

    match answer {
        Ok(choice) => {
            match choice {
                "New" => {
                    println!("Creating new empty simulator simulation!");
                    Simulator::new()
                },
                "Load" => {
                    let loaded_circuit = load();
                    Simulator::from_circuit(loaded_circuit)
                },
                _ => simple_error_exiting(),
            }
        },
        Err(_) => {
            simple_error_exiting();
        }
    }
}

fn menu(simulator: &mut Simulator) -> bool {

    let options = vec![
        "Add",
        "Interact",
        "Inspect",
        "Save",
        "Exit",
    ];

    let answer = Select::new("What do you want to do?", options).prompt();

    let choice = match answer {
        Ok(choice) => choice,
        Err(_) => simple_error_exiting(),
    };

    match choice {
        "Add" => add(simulator),
        "Interact" => interact(simulator),
        "Inspect" => inspect(simulator),
        "Save" => save(simulator),
        "Exit" => {
            println!("Exiting...");
            return false;
        },
        _ => simple_error_exiting(),
    }

    true
}

fn add(simulator: &mut Simulator) {

    let element_options = &[
        INPUT,
        OUTPUT,
        COMPONENT,
    ];

    let element_answer = Select::new("Which element should be added?", element_options.to_vec()).prompt();

    if let Ok(choice) = element_answer {

        match choice {
            INPUT => add_input(simulator),
            OUTPUT => add_output(simulator),
            COMPONENT => add_component(simulator),
            _ => (),
        }

    } else {
        simple_error();
    }
}

fn add_input(simulator: &mut Simulator) {

    let options = &[Value::On, Value::Off];

    let answer = Select::new("What should be the initial value of the new input?", options.to_vec()).prompt();

    match answer {
        Ok(choice) => {
            let (input_index, value_index) = simulator.add_input(choice);
            println!("New input with index {input_index} and initial value {choice} at value index {value_index} has been added.");
        },
        Err(_) => simple_error(),
    }
}

fn add_output(simulator: &mut Simulator) {

    if simulator.circuit().all_values().is_empty() {
        println!("The simulator has no values. Without a value no output can be added.");
        return;
    }

    let options: Vec<_> = (0..simulator.circuit().all_values().len()).collect();

    let answer = Select::new("Which value should the new output read?", options).prompt();

    match answer {
        Ok(choice) => {
            let output_index = simulator.add_output(choice);
            println!("New output with index {output_index} reading value form {choice} has been added.");
        },
        Err(_) => simple_error(),
    }
}

fn add_component(simulator: &mut Simulator) {

    let functions = &[
        Function::And,
        Function::Or,
        Function::Not,
    ];

    let applicable_functions: Vec<_> = functions.iter().filter(|function| function.input_value_count() <= simulator.circuit().all_values().len()).collect();

    if applicable_functions.is_empty() {
        println!("There are no components that can be created because there are to few values that could be used as inputs.");
        return;
    }

    let funtion_answer = Select::new("Which function should the new component be using?", applicable_functions).prompt();

    match funtion_answer {
        Ok(&function_choice) => {
            let input_value_indices: Vec<_> = (0..simulator.circuit().all_values().len()).collect();

            let valid_input_number = function_choice.input_value_count();
            let validator = move |a: &[ListOption<&usize>]| {
                if a.len() < valid_input_number {
                    Ok(Validation::Invalid("Too few input values selected.".into()))
                } else if a.len() > valid_input_number {
                    Ok(Validation::Invalid("Too many input values selected.".into()))
                } else {
                    Ok(Validation::Valid)
                }
            };

            let input_answer = MultiSelect::new("Choose the values to use as inputs for the component:", input_value_indices)
                .with_validator(validator)
                .prompt();

            match input_answer {
                Ok(input_choice) => {
                    let (component_index, output_indices) = simulator.add_component(function_choice, input_choice.clone());
                    println!("Component with index {component_index} using function {function_choice} on inputs {input_choice:?} with outputs {output_indices:?} has been added.")
                },
                Err(_) => simple_error(),
            }
        },
        Err(_) => simple_error(),
    }
}

fn interact(simulator: &mut Simulator) {

    let interaction_options = &[
        "Set input",
        "Simulate",
        "Step",
    ];

    let interaction_answer = Select::new("Which interaction do you want to do?", interaction_options.to_vec()).prompt();

    if let Ok(interaction_choice) = interaction_answer {
        match interaction_choice {
            "Set input" => set_input(simulator),
            "Simulate" => simulate(simulator),
            "Step" => simulate_step(simulator),
            _ => simple_error()
        }
    }
}

fn set_input(simulator: &mut Simulator) {
    let input_index_options = (0..simulator.circuit().all_inputs().len()).collect();
    let input_index_answer = Select::new("Which input value should be set?", input_index_options).prompt();

    if let Ok(input_index_choice) = input_index_answer {

        let value_options = &[
            Value::On,
            Value::Off,
        ];

        let new_value_answer = Select::new("What value should the input be set to?", value_options.to_vec()).prompt();

        if let Ok(new_value_choice) = new_value_answer {
            simulator.set_input(input_index_choice, new_value_choice);
            println!("The value of input {input_index_choice} has been set to {new_value_choice}.");
        }

    } else {
        simple_error();
    }
}

fn simulate(simulator: &mut Simulator) {
    if simulator.simulate() {
        println!("Simulation ran into stable condition.");
    } else {
        println!("Simulation finished in unstable condition.");
    }

    simulator.circuit().all_outputs().iter()
        .map(|output| simulator.value_for_output(output))
        .enumerate()
        .for_each(|(output_index, value)| println!("\tOutput {output_index} has value {value}."));
}

fn simulate_step(simulator: &mut Simulator) {
    simulator.step();
    println!("Stepped");

    simulator.circuit().all_outputs().iter()
        .map(|output| simulator.value_for_output(output))
        .enumerate()
        .for_each(|(output_index, value)| println!("\tOutput {output_index} has value {value}."));
}

fn inspect(simulator: &mut Simulator) {

    let inspect_options = &[
        INPUT,
        OUTPUT,
        COMPONENT,
        VALUE,
    ];

    let inspect_answer = Select::new("Which element should be inspected?", inspect_options.to_vec()).prompt();

    if let Ok(inspect_choice) = inspect_answer {

        let select_options = &[
            ALL,
            BY_INDEX,
        ];

        let select_answer = Select::new("Do you want to inspect all or by index?", select_options.to_vec()).prompt();

        if let Ok(select_choice) = select_answer {

            match select_choice {
                ALL => {
                    match inspect_choice {
                        INPUT => {
                            println!("Inspecting all inputs:");

                            simulator.circuit().all_inputs().iter().enumerate().for_each(|(i, input)| {
                                println!("Index: {i}\n{input:?}");
                            });
                        },
                        OUTPUT => {
                            println!("Inspecting all outputs:");

                            simulator.circuit().all_outputs().iter().enumerate().for_each(|(i, input)| {
                                println!("Index: {i}\n{input:?}");
                            });
                        },
                        COMPONENT => {
                            println!("Inspecting all components:");

                            simulator.circuit().all_components().iter().enumerate().for_each(|(i, input)| {
                                println!("Index: {i}\n{input:?}");
                            });
                        },
                        VALUE => {
                            println!("Inspecting all values:");

                            simulator.circuit().all_values().iter().enumerate().for_each(|(i, input)| {
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
                        INPUT => (0..simulator.circuit().all_inputs().len()).collect(),
                        OUTPUT => (0..simulator.circuit().all_outputs().len()).collect(),
                        COMPONENT => (0..simulator.circuit().all_components().len()).collect(),
                        VALUE => (0..simulator.circuit().all_values().len()).collect(),
                        _ => {
                            simple_error();
                            return;
                        },
                    };

                    let index_answer = Select::new("Which index should be shown?", choosable_indices).prompt();

                    if let Ok(index_choice) = index_answer {
                        match inspect_choice {
                            INPUT => {
                                let choosen_input = &simulator.circuit().all_inputs()[index_choice];
                                println!("Inspecting input at index {index_choice}:\n{choosen_input:?}");
                            },
                            OUTPUT => {
                                let choosen_input = &simulator.circuit().all_outputs()[index_choice];
                                println!("Inspecting output at index {index_choice}:\n{choosen_input:?}");
                            },
                            COMPONENT => {
                                let choosen_input = &simulator.circuit().all_components()[index_choice];
                                println!("Inspecting component at index {index_choice}:\n{choosen_input:?}");
                            },
                            VALUE => {
                                let choosen_input = &simulator.circuit().all_values()[index_choice];
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

fn save(simulator: &mut Simulator) {
    if let Ok(serialized_circuit) = serde_json::to_string(simulator.circuit()) {

        let current_dir = std::env::current_dir().unwrap();
        let help_message = format!("Current directory: {}", current_dir.to_string_lossy());

        let save_location_answer = Text::new("Save location:")
            .with_autocomplete(cli_util::FilePathCompleter::default())
            .with_help_message(&help_message)
            .prompt();

        if let Ok(save_location_choice) = save_location_answer {
            if let Ok(mut file) = fs::File::create(save_location_choice) {
                if file.write(serialized_circuit.as_bytes()).is_ok() {
                    println!("Circuit has been saved.");
                }
            }

        } else {
            println!("Error while choosing save location!");
        }

    } else {
        println!("Error while serializing!");
    }
}

fn load() -> Circuit {
    let current_dir = std::env::current_dir().unwrap();
    let help_message = format!("Current directory: {}", current_dir.to_string_lossy());

    let file_to_load_answer = Text::new("File to load:")
        .with_autocomplete(cli_util::FilePathCompleter::default())
        .with_help_message(&help_message)
        .prompt();

    if let Ok(file_to_load_choice) = file_to_load_answer {
        let serial_circuit = fs::read(file_to_load_choice).unwrap();
        let loaded_circuit: Circuit = serde_json::from_slice(&serial_circuit).unwrap();

        loaded_circuit
    } else {
        println!("Error while choosing save location!");
        Circuit::new()
    }
}

fn simple_error() {
    println!("There was an error!");
}

fn simple_error_exiting() -> ! {
    simple_error();
    exit(1)
}