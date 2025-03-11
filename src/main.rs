use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::process::Child;

// Function to read user input
fn read_input() -> String {
    print!("rush$ ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input.trim().to_string()
}

// Function to parse input into words and determine background execution
fn parse_input(input: &str) -> (Vec<&str>, bool) {
    let mut words: Vec<&str> = input.split_whitespace().collect();
    let mut background = false;
    
    if let Some(last_word) = words.last() {
        if last_word == &"&" {
            background = true;
            words.pop();
        }
    }

    (words, background)
}
fn handle_redirection(words: &mut Vec<&str>) -> (Option<String>, Option<String>, bool) {
    let mut stdin_redirection = None;
    let mut stdout_redirection = None;
    let mut stdout_append = false;

    let mut index = 0;
    while index < words.len() {
        let word = words[index];
        match word {
            "<" => {
                let _ = words.remove(index);
                if let Some(filename) = words.get(index).map(|&s| s.to_string()) {
                    stdin_redirection = Some(filename);
                    words.remove(index);
                } else {
                    panic!("No filename provided after '<'");
                }
            }
            ">" | ">>" => {
                let operator = words.remove(index);
                if let Some(filename) = words.get(index).map(|&s| s.to_string()) {
                    stdout_redirection = Some(filename);
                    stdout_append = operator == ">>";
                    words.remove(index);
                } else {
                    panic!("No filename provided after '{}' operator", operator);
                }
            }
            _ => {
                index += 1;
            }
        }
    }

    (stdin_redirection, stdout_redirection, stdout_append)
}



// Function to execute command
fn execute_command(words: Vec<&str>, stdin_redirection: Option<String>, stdout_redirection: Option<String>, stdout_append: bool, background: bool) -> Child {
    let mut command = Command::new(words[0]);
    command.args(&words[1..]);

    if let Some(filename) = stdin_redirection {
        command.stdin(Stdio::from(std::fs::File::open(&filename).expect("Failed to open file for stdin")));
    }

    if let Some(filename) = stdout_redirection {
        let file = if stdout_append {
            std::fs::OpenOptions::new().create(true).append(true).open(&filename).expect("Failed to open file for stdout")
        } else {
            std::fs::File::create(&filename).expect("Failed to create file for stdout")
        };
        command.stdout(Stdio::from(file));
    }

    if background {
        command.spawn().expect("Failed to execute command in background")
    } else {
        command.spawn().expect("Failed to execute command")
    }
}

// Function to handle background processes
fn handle_background_processes(background_processes: &mut Vec<Child>) {
    let mut i = 0;
    while i < background_processes.len() {
        if let Some(status) = background_processes[i].try_wait().expect("Failed to wait on child process") {
            if status.success() {
                background_processes.remove(i);
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
}

fn validate_command(command: &[&str]) -> bool {
    let mut iter = command.iter().enumerate();
    while let Some((_index, &word)) = iter.next() {
        match word {
            "<" | ">" | ">>" => {
                if let Some((_, &s)) = iter.next() {
                    let filename = s.to_owned();
                    if filename == ">" || filename == ">>" {
                        println!("{} must have a file afterwards!", word);
                        return true;
                    }
                } else {
                    println!("{} must have a file afterwards!", word);
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}
fn validate_command2(command: &[&str]) -> bool {
    let mut greater_than_present = false;
    let mut double_greater_than_present = false;

    for &word in command {
        match word {
            ">" => greater_than_present = true,
            ">>" => double_greater_than_present = true,
            _ => {}
        }
    }

    if greater_than_present && double_greater_than_present {
        println!("Can only have one > or >>!");
        true
    } else {
        false
    }
}

fn main() {
    let mut background_processes: Vec<Child> = Vec::new();
    
    loop {
        let input = read_input();
        if input == "exit" {
            break;
        }

        let (mut words, background) = parse_input(&input);
		if validate_command(&words) {
            // If command is invalid, skip the rest of the loop iteration
            continue;
        }
		if validate_command2(&words) {
			continue;
		}

        let (stdin_redirection, stdout_redirection, stdout_append) = handle_redirection(&mut words);
        let mut child = execute_command(words, stdin_redirection, stdout_redirection, stdout_append, background);

        if background {
            background_processes.push(child);
        } else {
            child.wait().expect("Failed to wait on child process");
        }

        handle_background_processes(&mut background_processes);
    }
}
