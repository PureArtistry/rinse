use std::{
    env::var,
    fs::{self, File},
    io::{BufRead, BufReader},
    net::TcpStream,
    path::Path
};

use anyhow::{bail, Result};
use eframe::egui::Color32;
use mpd::{Client, Song, Status};

pub fn startup() -> Result<(Client<TcpStream>, Status, String)> {
    let music_dir = find_music_dir()?;
    let mut client = get_client()?;
    let status = client.status()?;
    match status.queue_len > 1 {
        true => Ok((client, status, music_dir)),
        false => bail!("Not enough songs in the queue!")
    }
}

// TODO: add support for UnixStream client, this will require some restucturing of the app
fn get_client() -> Result<Client<TcpStream>> { Ok(Client::new(TcpStream::connect("127.0.0.1:6600")?)?) }

// mpd only allows directly reading the music directory from a (local) unix socket rather than TCP :(
pub fn find_music_dir() -> Result<String> {
    let bail_msg = "Unable to determine music directory root!";

    let prefix = var("XDG_CONFIG_HOME").unwrap_or_else(|_| [&var("HOME").unwrap(), ".config"].join("/"));
    let mpd_conf_path = Path::new(&[&prefix, "mpd", "mpd.conf"].join("/")).to_owned();
    let mpd_conf = File::open(mpd_conf_path)?;

    for x in BufReader::new(mpd_conf).lines().flatten() {
        if x.starts_with("music_directory") {
            let value_vec = x.split_whitespace().collect::<Vec<_>>();
            let mut value = value_vec[1].to_owned();
            value.remove(0);
            value.remove(value.len() - 1);
            if value.starts_with('/') {
                return Ok(value.to_owned())
            }
            else if value.starts_with('~') {
                value.remove(0);
                return Ok([var("HOME").unwrap(), value].join(""))
            }
            else {
                bail!(bail_msg)
            }
        }
    }
    bail!(bail_msg)
}

pub fn gen_theme(path: &str) {
    let theme = "scheme: \"Nord\"
author: \"arcticicestudio\"
base00: \"2E3440\"
base01: \"3B4252\"
base02: \"434C5E\"
base03: \"4C566A\"
base04: \"D8DEE9\"
base05: \"E5E9F0\"
base06: \"ECEFF4\"
base07: \"8FBCBB\"
base08: \"88C0D0\"
base09: \"81A1C1\"
base0A: \"5E81AC\"
base0B: \"BF616A\"
base0C: \"D08770\"
base0D: \"EBCB8B\"
base0E: \"A3BE8C\"
base0F: \"B48EAD\"
";

    let folder = Path::new(&[path, "rinse"].join("/")).to_owned();
    if !folder.exists() {
        fs::create_dir(folder).expect("error: Can't create config directory!")
    }
    fs::write(Path::new(&[path, "rinse", "theme.yaml"].join("/")), theme)
        .expect("error:: Can't write theme file to config directory!")
}

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
