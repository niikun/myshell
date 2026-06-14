#[allow(unused_imports)] 
use std::io;
use std::io::{Write};
use std::process::Command;
use std::path;
use std::fs::File;
use std::os::unix::fs::PermissionsExt;

enum Redirect{
    Stdout(String),
    StdoutAppend(String),
}

impl Redirect {
    fn new(){
    }
}

struct CommandsInfo {
    commands:String,
    args:Vec<String>,
    redirect:Option<Redirect>,
}

impl CommandsInfo {
    fn new(inputs:String)-> Self{
        // let mut inputs_ = inputs.split_whitespace();
        let mut input_ =String::new();
        let mut messages  = Vec::new();
        let mut is_double_quoted = false;
        let mut is_single_quoted:bool = false;
        let mut is_redirected = false;
        let mut is_redirect_updated = false;
        let mut redirect_file:String = String::new();
        let mut redirect :Option<Redirect> = None; 
        for c  in inputs.chars(){
            match c {
                '>' => {
                    if is_redirected{
                        is_redirected = false;
                        is_redirect_updated = true;
                    } else {
                        is_redirected = true;
                    }
                },
                '\"' => {
                    is_double_quoted = !is_double_quoted;
                },
                '\'' => {
                    is_single_quoted = !is_single_quoted;
                },                        
                '$' if is_single_quoted => input_.push(c),
                '\n' => {
                    if is_redirected{
                        is_redirected = !is_redirected;   
                        redirect = Some(Redirect::Stdout(redirect_file.clone()));
                    } else if is_redirect_updated{
                        is_redirect_updated = false;
                        redirect = Some(Redirect::StdoutAppend(redirect_file.clone()));
                    }
                },
                ' ' if !is_double_quoted & !is_single_quoted => {
                        if !input_.is_empty() && !is_redirected && !is_redirect_updated{
                            messages.push(input_.clone());
                            input_.clear();
                        }
                    },
                _  =>{
                    if is_redirected | is_redirect_updated {
                        redirect_file.push(c);
                    } else {
                        input_.push(c)
                    }
                } ,
            }
        }
        messages.push(input_);

        let mut message_iter = messages.iter();
        let commands: String= message_iter.next().unwrap().trim().to_string();
        let mut args:Vec<String> = message_iter.map(|x| x.to_string()).collect();
        Self{
            commands:commands,
            args:args,
            redirect:redirect,
        }
    }
}
        
const BUILTIN_COMMANDS: [&str; 4] = ["echo", "exit", "pwd", "type"];

fn main() {
    loop{
        let path_env = std::env::var("PATH").unwrap_or("".to_string());
        print!("$ ");
        io::stdout().flush().unwrap();
        // io::stdout().flush().unwrap();
        let mut inputs:String = String::new();
        io::stdin().read_line(&mut inputs).unwrap();
        let commands_info = CommandsInfo::new(inputs);
        let path_iter = path_env.split(":");
        match commands_info.commands.as_str() {
            "" => continue,
            "cd" => {
                let destination = commands_info.args.get(0).map_or("",|v|v);
                let mut current_dir  = std::env::current_dir().unwrap();
                match destination {
                    "~" | "" => std::env::set_current_dir(std::env::var("HOME").unwrap()).unwrap(),
                    ".." => {
                        current_dir.pop();
                        std::env::set_current_dir(current_dir).unwrap();
                    },
                    _ => {
                        current_dir.push(destination);
                        if let Err(_) = std::env::set_current_dir(&current_dir) {
                            println!("cd: {}: No such file or directory", destination);
                        }  
                    },
                }
            }
            "exit" => break,
            "echo" => {
                println!("{}",commands_info.args.join(" ").trim());
            },
            "pwd" => {
                let current_dir = std::env::current_dir();
                match current_dir{
                    Ok(dir) => println!("{}",dir.to_string_lossy()),
                    Err(_) => println!(""),
                }
            }
            "type" => {
                let cmd = commands_info.args.get(0).map_or("",|v|v);
                let type_args = commands_info.args.get(1..).unwrap_or(&[]);
                if let Some(file) = BUILTIN_COMMANDS.iter().find(|s|s.to_string()==*cmd){
                    println!("{} is a shell builtin.",file);
                } else{
                    if let Some(dir) = path_iter
                            .map(|p| p.to_string() + "/" + &cmd)
                            .find(|dir| path::Path::new(dir).exists()){
                                if std::fs::metadata(&dir).unwrap().permissions().mode() & 0o111 != 0 {
                                    println!("{} is {}", cmd, dir);
                                } else {
                                    println!("{}: not allowed", cmd);
                                }                                    
                    } else {
                        println!("{}: not found", cmd);
                    }
                }
            },

            _ => {
                let mut is_in_path = false;
                path_env.split(":").find_map(|dir|{
                    let command = dir.to_string() + "/" + &commands_info.commands;
                    let child = Command::new(command)
                                    .args(commands_info.args.clone())
                                    .spawn();


                    match child {
                        Ok(mut result) => {
                            is_in_path = true;
                            result.wait().unwrap();
                            Some(())
                        },
                        Err(_) => None,
                    }
                });
               
                if !is_in_path{
                    println!("{} is not found.",commands_info.commands)
                }
            }
                
        }
    }
}