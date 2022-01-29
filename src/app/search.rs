use asearch::Asearch;
use edit_distance::edit_distance;
use mpd::Song;

use super::{imp::SearchResult, utils};

pub fn build_list(q: &str, queue: &[Song]) -> Vec<SearchResult> {
    let mut list = vec![];
    match q.is_empty() {
        true => {
            for (i, song) in queue.iter().enumerate() {
                list.push(SearchResult {
                    title: utils::gen_title(song),
                    pos:   i,
                    ed:    0
                })
            }
        }

        false => {
            let query = Asearch::new([" ", q, " "].join(""));
            let max_ed = 99;

            for (i, song) in queue.iter().enumerate() {
                let match_title = match song.title.is_some() {
                    true => song.title.as_ref().unwrap(),
                    false => &song.file
                };
                let title = utils::gen_title(song);

                if query.find(&*match_title, 0) {
                    let ed = edit_distance(q, match_title);
                    if ed < max_ed {
                        list.push(SearchResult { title, pos: i, ed })
                    }
                }
                else {
                    let tags = &song.tags;
                    let album = tags.get("Album").map(|x| x.to_owned());
                    let artist = tags.get("Artist").map(|x| x.to_owned());

                    if let Some(x) = album {
                        if query.find(&x, 0) {
                            let ed = edit_distance(q, &x);
                            if ed < max_ed {
                                list.push(SearchResult {
                                    title,
                                    pos: i,
                                    ed: ed + 100
                                });
                                continue
                            }
                        }
                    }
                    if let Some(x) = artist {
                        if query.find(&x, 0) {
                            let ed = edit_distance(q, &x);
                            if ed < max_ed {
                                list.push(SearchResult {
                                    title,
                                    pos: i,
                                    ed: ed + 200
                                });
                            }
                        }
                    }
                }
            }
            list.sort_unstable_by(|a, b| a.ed.cmp(&b.ed));
        }
    }
    list
}
