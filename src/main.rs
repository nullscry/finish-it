// use rusqlite::NO_PARAMS;
use rusqlite::{Connection, Result};
use std::{fs, io::Error, io::ErrorKind, path};

fn parse_args(args: Vec<String>) -> Result<(), Error> {
    if args.len() == 0 {
        return Err(Error::new(ErrorKind::Other, "Usage"));
    }

    match &args[..] {
        [command, rest @ ..] => {
            println!("command found {} with args {:?}", command, rest);
        }
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
    match parse_args(args) {
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
