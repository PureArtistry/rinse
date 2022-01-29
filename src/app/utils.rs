use std::{net::TcpStream, path::Path};

use anyhow::{bail, Result};
use eframe::egui::Color32;
use mpd::{Client, Song, Status};

pub fn startup() -> Result<(Client<TcpStream>, Status)> {
    let mut client = get_client()?;
    let status = client.status()?;
    match status.queue_len > 1 {
        true => Ok((client, status)),
        false => bail!("Not enough songs in the queue!")
    }
}

fn get_client() -> Result<Client<TcpStream>> { Ok(Client::new(TcpStream::connect("127.0.0.1:6600")?)?) }

pub fn gen_title(song: &Song) -> String {
    match &song.title {
        Some(x) => x.to_owned(),
        None => Path::new(&song.file)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }
}

pub fn gen_switcher(cycle: u8, status: &Status, queue: &[Song]) -> String {
    match cycle {
        0 => ["聾  ", &gen_title(&queue[status.song.unwrap().pos as usize])].join(""),
        1 => ["  ", &gen_title(&queue[status.song.unwrap().pos as usize])].join(""),
        2 => ["嶺  ", &gen_title(&queue[status.nextsong.unwrap().pos as usize])].join(""),
        3 => [
            "墳 ",
            &status.volume.to_string(),
            "  凌 ",
            state_string(status.repeat),
            "  咽 ",
            state_string(status.random),
            "  綾 ",
            state_string(status.single),
            "  裸 ",
            state_string(status.consume)
        ]
        .join(""),
        _ => unreachable!()
    }
}

pub fn gen_colour(s: &str) -> Color32 {
    let rgb = (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect::<Vec<u8>>();
    Color32::from_rgb(rgb[0], rgb[1], rgb[2])
}

pub fn time_string(s: i64) -> String {
    let mut r = vec![];

    let mins = s / 60;
    let hrs = mins / 60;
    let days = hrs / 24;

    match days > 0 {
        true if days == 1 => r.push([days.to_string(), " day, ".to_string()].join("")),
        true => r.push([days.to_string(), " days, ".to_string()].join("")),
        false => {}
    }

    match hrs > 0 {
        true if hrs == 1 => r.push([(hrs % 24).to_string(), " hour, ".to_string()].join("")),
        true => r.push([(hrs % 24).to_string(), " hours, ".to_string()].join("")),
        false => {}
    }

    match mins > 0 {
        true if mins == 1 => r.push([(mins % 60).to_string(), " minute, ".to_string()].join("")),
        true => r.push([(mins % 60).to_string(), " minutes, ".to_string()].join("")),
        false => {}
    }

    match s % 60 == 1 {
        true => r.push([(s % 60).to_string(), " second".to_string()].join("")),
        false => r.push([(s % 60).to_string(), " seconds".to_string()].join(""))
    }

    r.join("")
}

pub fn progress_string(e: i64, d: i64) -> String {
    if d == 0 {
        return "".to_string()
    }

    let mut e_mins = (e / 60000).to_string();
    let mut e_secs = ((e / 1000) % 60).to_string();
    let d_mins = (d / 60000).to_string();
    let mut d_secs = ((d / 1000) % 60).to_string();

    let mut r = vec!["[ "];

    while e_mins.len() < d_mins.len() {
        e_mins.insert(0, '0')
    }
    if e_secs.len() == 1 {
        e_secs.insert(0, '0')
    }
    if d_secs.len() == 1 {
        d_secs.insert(0, '0')
    }
    let e_joined = [e_mins, ":".to_string(), e_secs].join("");
    let d_joined = [d_mins, ":".to_string(), d_secs].join("");

    r.push(&e_joined);
    r.push(" - ");
    r.push(&d_joined);
    r.push(" ]");
    r.join("")
}

fn state_string(s: bool) -> &'static str {
    if s {
        "on"
    }
    else {
        "off"
    }
}
