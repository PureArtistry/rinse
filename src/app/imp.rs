use std::{collections::BTreeMap, env::var, fs, net::TcpStream, path::Path, time::Instant};

use eframe::{
    egui::{Color32, TextureId, Vec2},
    epi::Image
};
use mpd::{Client, Song, State, Status};

use super::{images, search, utils};

pub struct Rinse {
    pub data: Data
}

pub struct Data {
    pub colours:          Colours,
    pub update_timer:     Option<Instant>,
    pub mpc:              Client<TcpStream>,
    pub paths:            (String, String),
    pub queue:            Vec<Song>,
    pub current_pos:      usize,
    pub state:            State,
    pub showing_info:     usize,
    pub cover:            Option<(Vec2, TextureId)>,
    pub info_title:       Option<String>,
    pub info_artist:      Option<String>,
    pub info_album:       Option<String>,
    pub info_duration:    Option<String>,
    pub info_date:        Option<String>,
    pub elapsed:          Option<i64>,
    pub duration:         Option<i64>,
    pub switcher:         String,
    pub switcher_timer:   Option<Instant>,
    pub switcher_cycle:   u8,
    pub search_query:     String,
    pub list:             Vec<SearchResult>,
    pub selected:         usize,
    pub selected_pos:     usize,
    pub interacted:       bool,
    pub need_list_scroll: bool
}

pub trait Setup {
    fn setup(stuff: (Client<TcpStream>, Status)) -> Self;
}

impl Setup for Rinse {
    fn setup(stuff: (Client<TcpStream>, Status)) -> Self {
        let (mut mpc, status) = stuff;
        let music_dir = var("XDG_MUSIC_DIR").expect(
            "error: music directory can't be obtained from mpd!\nyou need to set your XDG_MUSIC_DIR \
             environment variable to match the value of music_directory from your mpd.conf"
        );
        let queue = mpc.queue().unwrap();

        let current_pos = status.song.unwrap().pos as usize;
        let elapsed = status.elapsed.map(|x| x.to_owned().num_milliseconds());
        let duration = status.duration.map(|x| x.to_owned().num_milliseconds());

        let song = SongInfo::update(&music_dir, &queue[current_pos]);

        let switcher_cycle = match status.nextsong.is_some() {
            true => 2,
            false => 3
        };
        let switcher = utils::gen_switcher(switcher_cycle, &status, &queue);

        let search_query = String::new();
        let list = search::build_list(&search_query, &queue);

        let data = Data {
            colours: Colours::default(),
            update_timer: None,
            mpc,
            paths: (music_dir, song.filepath),
            queue,
            current_pos,
            state: status.state,
            showing_info: current_pos,
            cover: None,
            info_title: Some(song.title),
            info_artist: song.artist,
            info_album: song.album,
            info_duration: song.duration,
            info_date: song.date,
            elapsed,
            duration,
            switcher,
            switcher_timer: None,
            switcher_cycle,
            search_query,
            list,
            selected: current_pos,
            selected_pos: current_pos,
            interacted: false,
            need_list_scroll: true
        };
        Self { data }
    }
}

pub struct SongInfo {
    pub cover:    Image,
    pub title:    String,
    pub artist:   Option<String>,
    pub album:    Option<String>,
    pub duration: Option<String>,
    pub date:     Option<String>,
    pub filepath: String
}

pub trait Update {
    fn update(music_dir: &str, song: &Song) -> Self;
}

impl Update for SongInfo {
    fn update(music_dir: &str, song: &Song) -> Self {
        let tags = &song.tags;
        Self {
            cover:    images::get_cover(&(music_dir, &song.file)),
            title:    utils::gen_title(song),
            artist:   tags.get("Artist").map(|x| x.to_owned()),
            album:    tags.get("Album").map(|x| x.to_owned()),
            duration: song
                .duration
                .as_ref()
                .map(|x| utils::time_string(x.num_seconds())),
            date:     tags.get("Date").map(|x| x.to_owned()),
            filepath: song.file.to_owned()
        }
    }
}

pub struct SearchResult {
    pub title: String,
    pub pos:   usize,
    pub ed:    usize
}

#[allow(non_snake_case)]
pub struct Colours {
    pub base00: Color32,
    pub base01: Color32,
    pub base02: Color32,
    pub base03: Color32,
    pub base04: Color32,
    pub base05: Color32,
    pub base06: Color32,
    pub base07: Color32,
    pub base08: Color32,
    pub base09: Color32,
    pub base0A: Color32,
    pub base0B: Color32,
    pub base0C: Color32,
    pub base0D: Color32,
    pub base0E: Color32,
    pub base0F: Color32
}

impl Default for Colours {
    fn default() -> Self {
        let prefix = var("XDG_CONFIG_HOME").unwrap_or_else(|_| [&var("HOME").unwrap(), ".config"].join("/"));
        let theme_path = Path::new(&[&prefix, "rinse", "theme.yaml"].join("/")).to_owned();
        let colours: BTreeMap<String, String> =
            serde_yaml::from_str(&fs::read_to_string(theme_path).unwrap()).unwrap();
        Self {
            base00: utils::gen_colour(colours.get("base00").unwrap()),
            base01: utils::gen_colour(colours.get("base01").unwrap()),
            base02: utils::gen_colour(colours.get("base02").unwrap()),
            base03: utils::gen_colour(colours.get("base03").unwrap()),
            base04: utils::gen_colour(colours.get("base04").unwrap()),
            base05: utils::gen_colour(colours.get("base05").unwrap()),
            base06: utils::gen_colour(colours.get("base06").unwrap()),
            base07: utils::gen_colour(colours.get("base07").unwrap()),
            base08: utils::gen_colour(colours.get("base08").unwrap()),
            base09: utils::gen_colour(colours.get("base09").unwrap()),
            base0A: utils::gen_colour(colours.get("base0A").unwrap()),
            base0B: utils::gen_colour(colours.get("base0B").unwrap()),
            base0C: utils::gen_colour(colours.get("base0C").unwrap()),
            base0D: utils::gen_colour(colours.get("base0D").unwrap()),
            base0E: utils::gen_colour(colours.get("base0E").unwrap()),
            base0F: utils::gen_colour(colours.get("base0F").unwrap())
        }
    }
}
