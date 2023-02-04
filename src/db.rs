use rusqlite::{Connection, Result};
use std::path;
use tui::widgets::StatefulWidget;

use super::add::TextAreaContainer;
use crate::{EventItem, InstanceItem};
pub fn get_db_connection() -> Connection {
    let db_path = path::Path::new("var/fit.db");

    if !path::Path::exists(db_path) {
        panic!(
            "Problem opening the file: {db_path:?}\nExecute \"fitdb create\" to initialize database at {db_path:?}"
        );
    }

    Connection::open(db_path).unwrap()
}

pub fn read_events_from_db(conn: &Connection) -> Result<Vec<EventItem>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT * FROM events")?;
    let event_iter = stmt.query_map([], |row| {
        Ok(EventItem {
            name: row.get(0)?,
            created: row.get(1)?,
        })
    })?;

    let mut events = Vec::new();
    for event in event_iter {
        events.push(event?);
    }

    Ok(events)
}

pub fn read_instances_count_from_db(
    conn: &Connection,
    selected_event: &str,
) -> Result<usize, rusqlite::Error> {
    let mut stmt = conn.prepare(
        format!("SELECT COUNT(*) FROM instances WHERE eventtype = \"{selected_event}\"").as_str(),
    )?;
    let mut rows = (stmt.query([]))?;

    let row = rows.next().unwrap().expect("Invalid Event");
    let instance_count: usize = row.get(0).expect("Invalid Event");

    Ok(instance_count)
}

pub fn read_instances_from_db(
    conn: &Connection,
    event_name: &str,
) -> Result<Vec<InstanceItem>, rusqlite::Error> {
    let mut stmt = conn
        .prepare(format!("SELECT * FROM instances WHERE eventtype = \"{event_name}\"").as_str())?;
    let instance_iter = stmt.query_map([], |row| {
        Ok(InstanceItem {
            instanceid: row.get(0)?,
            name: row.get(1)?,
            eventtype: row.get(2)?,
            isrecurring: row.get(3)?,
            percentage: row.get(4)?,
            timesfinished: row.get(5)?,
            daylimit: row.get(6)?,
            created: row.get(7)?,
        })
    })?;

    let mut instances = Vec::new();
    for instance in instance_iter {
        instances.push(instance?);
    }

    Ok(instances)
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
        "INSERT OR IGNORE INTO events (name) VALUES (?1)",
        (texts[0],),
    )?;

    conn.execute(
        "INSERT INTO instances (name, eventtype, isrecurring, percentage, timesfinished, daylimit) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
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

pub fn update_instance(conn: &Connection, instance: &InstanceItem) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE instances \
        SET percentage = ?2, \
            timesfinished = ?3 \
        WHERE \
            instanceid = ?1;",
        (
            instance.instanceid,
            instance.percentage,
            instance.timesfinished,
        ),
    )?;
    Ok(())
}

pub fn delete_instance(conn: &Connection, instance: &InstanceItem) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE \
        FROM instances \
        WHERE instanceid = ?1",
        (instance.instanceid,),
    )?;
    Ok(())
}

pub fn delete_event(conn: &Connection, event: &EventItem) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE \
        FROM events \
        WHERE name = ?1",
        (&event.name,),
    )?;
    Ok(())
}
