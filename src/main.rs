use std::process::exit;

use inquire::Select;
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
        "New element",
        "Show all inputs",
        "Show all outputs",
        "Show all elements",
        "Read Input",
        "Read Output",
        "Read Value",
        "Exit",
    ];

    let answer = Select::new("What do you want to do?", options).prompt();

    let choice = match answer {
        Ok(choice) => choice,
        Err(_) => simple_error_exiting(),
    };

    match choice {
        "New input" => true,
        "New output" => true,
        "New element" => true,
        "Show all inputs" => true,
        "Show all outputs" => true,
        "Show all elements" => true,
        "Read Input" => true,
        "Read Output" => true,
        "Read Value" => true,
        "Exit" => {
            println!("Exiting...");
            false
        },
        _ => simple_error_exiting(),
    }
}

fn simple_error_exiting() -> ! {
    println!("There was an error!");
    exit(1)
}