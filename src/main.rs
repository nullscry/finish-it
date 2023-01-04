use rusqlite::NO_PARAMS;
use rusqlite::{Connection, Result};
use std::process::Command;
use std::{fs, io::Error, io::ErrorKind, path};

// fn split_args(args: Vec<String>) -> Result<(String, Vec<String>), Error> {
//     let first = match args.pop() {
//         Some(v) => v,
//         None => return Err(Error::new(ErrorKind::Other, "Usage")),
//     };

//     Ok((first, args))
// }

fn parse_args(args: Vec<String>, conn: Connection) -> Result<(), Error> {
    if args.len() == 0 {
        return Err(Error::new(ErrorKind::Other, "Usage"));
    }

    // args.reverse();

    let cmd = args[0].as_str();
    let num = &args[2];

    match &args[..] {
        [cmd, rest @ ..] => match cmd.as_str() {
            "list" => println!("Command list with args {:?}", rest),
            "create" => println!("Command create with args {:?}", rest),
            "delete" => println!("Command delete with args {:?}", rest),
            _ => eprintln!("Unknown command"),
        },
        _ => {
            return Err(Error::new(ErrorKind::Other, "Usage"));
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let db_path = path::Path::new("var/fit.db");

    if !path::Path::exists(db_path) {
        panic!(
            "Problem opening the file: {0:?}\nExecute \"fitdb create\" to initialize database at {0:?}",
            db_path
        );
    }

    let conn = Connection::open(db_path)?;

    let args: Vec<String> = std::env::args().skip(1).collect();
    match parse_args(args, conn) {
        Ok(_v) => {
            return Ok(());
        }
        Err(e) => {
            panic!("Error Encountered: {}", e)
        }
    }
}

// TODO
// - MATCH ARGUMENTS INCREMENTELY
