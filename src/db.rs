use rusqlite::{Connection, Result};
use std::path;

use super::add::TextAreaContainer;
use crate::{Item, Topic};
pub fn get_db_connection() -> Connection {
    let db_path = path::Path::new("var/fit.db");

    if !path::Path::exists(db_path) {
        panic!(
            "Problem opening the file: {db_path:?}\nExecute \"fitdb create\" to initialize database at {db_path:?}"
        );
    }

    Connection::open(db_path).unwrap()
}

pub fn read_topics_from_db(conn: &Connection) -> Result<Vec<Topic>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT * FROM topics")?;
    let event_iter = stmt.query_map([], |row| {
        Ok(Topic {
            name: row.get(0)?,
            created: row.get(1)?,
        })
    })?;

    let mut topics = Vec::new();
    for event in event_iter {
        topics.push(event?);
    }

    Ok(topics)
}

pub fn read_items_count_from_db(
    conn: &Connection,
    selected_event: &str,
) -> Result<usize, rusqlite::Error> {
    let mut stmt = conn.prepare(
        format!("SELECT COUNT(*) FROM items WHERE topicname = \"{selected_event}\"").as_str(),
    )?;
    let mut rows = (stmt.query([]))?;

    let row = rows.next().unwrap().expect("Invalid Event");
    let item_count: usize = row.get(0).expect("Invalid Event");

    Ok(item_count)
}

pub fn read_items_from_db(
    conn: &Connection,
    event_name: &str,
) -> Result<Vec<Item>, rusqlite::Error> {
    let mut stmt =
        conn.prepare(format!("SELECT * FROM items WHERE topicname = \"{event_name}\"").as_str())?;
    let item_iter = stmt.query_map([], |row| {
        Ok(Item {
            id: row.get(0)?,
            name: row.get(1)?,
            topicname: row.get(2)?,
            isrecurring: row.get(3)?,
            percentage: row.get(4)?,
            timesfinished: row.get(5)?,
            daylimit: row.get(6)?,
            created: row.get(7)?,
        })
    })?;

    let mut items = Vec::new();
    for item in item_iter {
        items.push(item?);
    }

    Ok(items)
}

pub fn insert_into_db(
    conn: &Connection,
    text_areas: &mut [TextAreaContainer],
) -> Result<(), rusqlite::Error> {
    let default = String::from("0");
    let texts: Vec<&str> = text_areas
        .iter()
        .map(|ta| ta.text_area.lines().get(0).unwrap_or(&default).trim())
        .collect();

    conn.execute(
        "INSERT OR IGNORE INTO topics (name) VALUES (?1)",
        (texts[0],),
    )?;

    conn.execute(
        "INSERT INTO items (name, topicname, isrecurring, percentage, timesfinished, daylimit) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        (texts[1], texts[0], texts[2], texts[3], texts[4], texts[5]),
    )?;

    text_areas.iter_mut().for_each(|ta| {
        ta.clear_text();
        ta.inactivate();
        ta.validate();
    });

    text_areas[0].activate();

    Ok(())
}

pub fn update_item(conn: &Connection, item: &Item) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE items \
        SET percentage = ?2, \
            timesfinished = ?3 \
        WHERE \
            id = ?1;",
        (item.id, item.percentage, item.timesfinished),
    )?;
    Ok(())
}

pub fn delete_item(conn: &Connection, item: &Item) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE \
        FROM items \
        WHERE id = ?1",
        (item.id,),
    )?;
    Ok(())
}

pub fn delete_topic(conn: &Connection, event: &Topic) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE \
        FROM topics \
        WHERE name = ?1",
        (&event.name,),
    )?;
    Ok(())
}
