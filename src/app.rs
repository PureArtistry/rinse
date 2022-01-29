mod images;
mod imp;
mod search;
pub mod utils;

use std::{net::TcpStream, time::Instant};

use eframe::{
    egui::{
        style::Selection, Align, CentralPanel, CtxRef, FontData, FontDefinitions, FontFamily, Key, Layout,
        RichText, ScrollArea, SidePanel, Slider, Stroke, TextEdit, TextStyle, Vec2
    },
    epi,
    epi::Frame,
    run_native, NativeOptions
};
use mpd::{Client, State, Status};

use self::imp::{Rinse, Setup, SongInfo, Update};

pub fn start(stuff: (Client<TcpStream>, Status)) {
    let options = NativeOptions {
        always_on_top:         true,
        maximized:             false,
        decorated:             false,
        drag_and_drop_support: false,
        icon_data:             None,
        initial_window_size:   Some([720.0, 600.0].into()),
        resizable:             false,
        transparent:           false
    };
    run_native(Box::new(Rinse::setup(stuff)), options)
}

impl epi::App for Rinse {
    fn name(&self) -> &str { "rinse" }

    fn warm_up_enabled(&self) -> bool { true }

    fn setup(&mut self, ctx: &CtxRef, frame: &Frame, _storage: Option<&dyn epi::Storage>) {
        let Self { data } = self;

        let mut fonts = FontDefinitions::default();

        fonts.font_data.insert(
            "font1".to_owned(),
            FontData::from_static(include_bytes!("../assets/font1.ttf")) // Victor Mono Italic Nerd Font
        );
        fonts
            .fonts_for_family
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "font1".to_owned());

        fonts.font_data.insert(
            "font2".to_owned(),
            FontData::from_static(include_bytes!("../assets/font2.ttf")) // Iosevka Regular Nerd Font
        );
        fonts
            .fonts_for_family
            .entry(FontFamily::Monospace)
            .or_default()
            .insert(0, "font2".to_owned());

        fonts
            .family_and_size
            .insert(TextStyle::Heading, (FontFamily::Proportional, 25.0));
        fonts
            .family_and_size
            .insert(TextStyle::Body, (FontFamily::Proportional, 25.0));
        fonts
            .family_and_size
            .insert(TextStyle::Monospace, (FontFamily::Monospace, 19.0));
        fonts
            .family_and_size
            .insert(TextStyle::Small, (FontFamily::Monospace, 16.0));
        fonts
            .family_and_size
            .insert(TextStyle::Button, (FontFamily::Monospace, 40.0));

        ctx.set_fonts(fonts);

        let (music_dir, filepath) = &data.paths;
        let image = images::get_cover(&(music_dir, filepath));
        let texture = frame.alloc_texture(image);
        let size = [250.0, 250.0].into();
        data.cover = Some((size, texture));

        data.switcher_timer = Some(Instant::now());
        data.update_timer = Some(Instant::now())
    }

    fn update(&mut self, ctx: &CtxRef, frame: &Frame) {
        let Self { data } = self;
        let input = ctx.input();

        if input.key_pressed(Key::Escape) || input.key_released(Key::Escape) {
            frame.quit()
        }
        if input.key_pressed(Key::Enter) && data.mpc.switch(data.selected_pos as u32).is_ok() {
            frame.quit()
        }

        if input.modifiers.ctrl && input.key_pressed(Key::U) {
            data.search_query = String::new();
            data.selected = data.current_pos;
            data.selected_pos = data.current_pos;
            data.need_list_scroll = true;
            data.interacted = false
        }

        if input.pointer.any_pressed() {
            data.interacted = true
        }

        if input.scroll_delta[1] != 0.0 && !data.interacted {
            data.need_list_scroll = false
        }

        if !data.list.is_empty() && input.key_pressed(Key::Tab) {
            match input.modifiers.shift {
                true => {
                    if data.selected == 0 {
                        data.selected = data.list.len() - 1
                    }
                    else {
                        data.selected -= 1
                    }
                    data.need_list_scroll = true;
                    data.interacted = true
                }
                false => {
                    if data.selected == data.list.len() - 1 {
                        data.selected = 0
                    }
                    else {
                        data.selected += 1
                    }
                    data.need_list_scroll = true;
                    data.interacted = true
                }
            }
        }

        if data.update_timer.as_ref().unwrap().elapsed().as_millis() > 33 {
            if let Ok(status) = data.mpc.status() {
                let current_pos = status.song.unwrap().pos as usize;
                let more = match status.nextsong.is_some() {
                    true => 2,
                    false => 3
                };

                if current_pos != data.current_pos {
                    let mut cycle = None;
                    if !data.interacted {
                        data.selected = current_pos;
                        data.selected_pos = current_pos;
                        cycle = Some(more);
                    }

                    data.duration = status.duration.map(|x| x.to_owned().num_milliseconds());

                    data.switcher_cycle = cycle.unwrap_or_else(|| match data.selected == current_pos {
                        true => more,
                        false if more == 2 => 1,
                        false => 0
                    });
                    data.switcher_timer = Some(Instant::now());
                    data.switcher = utils::gen_switcher(data.switcher_cycle, &status, &data.queue);

                    data.current_pos = current_pos
                }

                if data.switcher_timer.as_ref().unwrap().elapsed().as_secs() > 4 {
                    let next = match data.switcher_cycle {
                        0 => 3,
                        1 => 2,
                        2 => 3,
                        3 => match data.selected == data.current_pos {
                            true => 2,
                            false if more == 2 => 1,
                            false => 0
                        },
                        _ => unreachable!()
                    };
                    data.switcher = utils::gen_switcher(next, &status, &data.queue);
                    data.switcher_cycle = next;
                    data.switcher_timer = Some(Instant::now())
                }

                data.state = status.state;
                data.elapsed = status.elapsed.map(|x| x.to_owned().num_milliseconds());
            }
            data.update_timer = Some(Instant::now())
        }

        if data.showing_info != data.selected_pos {
            let song = SongInfo::update(&data.paths.0, &data.queue[data.selected_pos]);

            data.info_title = Some(song.title);
            data.info_artist = song.artist;
            data.info_album = song.album;
            data.info_duration = song.duration;
            data.info_date = song.date;

            frame.free_texture(data.cover.unwrap().1);
            let texture = frame.alloc_texture(song.cover);
            let size = [250.0, 250.0].into();
            data.cover = Some((size, texture));

            data.showing_info = data.selected_pos
        }

        SidePanel::right("info_panel")
            .resizable(false)
            .min_width(320.0)
            .max_width(320.0)
            .frame(eframe::egui::containers::Frame {
                margin: Vec2::new(10.0, 10.0),
                corner_radius: 0.0,
                fill: data.colours.base00,
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.visuals_mut().dark_mode = true;
                ui.spacing_mut().item_spacing = Vec2::new(0.0, 0.0);
                ui.spacing_mut().slider_width = 200.0;
                ui.add_space(20.0);
                if let Some((size, texture)) = data.cover {
                    ui.horizontal_top(|ui| {
                        ui.add_space(15.0);
                        match data.selected == data.current_pos {
                            true => {
                                ui.visuals_mut().widgets.noninteractive.bg_stroke.color = data.colours.base09
                            }
                            false => {
                                ui.visuals_mut().widgets.noninteractive.bg_stroke.color = data.colours.base0F
                            }
                        }
                        ui.group(|ui| ui.vertical(|ui| ui.image(texture, size)))
                    });
                }
                ui.add_space(20.0);

                ui.vertical(|ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.set_max_width(300.0);
                        ui.label(
                            RichText::new(data.info_title.as_ref().unwrap().to_string())
                                .heading()
                                .color(data.colours.base05)
                        )
                    });
                    ui.add_space(18.0);

                    let line_colour = match data.selected == data.current_pos {
                        true => data.colours.base09,
                        false => data.colours.base0F
                    };

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("ﴁ ").monospace().color(data.colours.base04));
                        ui.label(RichText::new("▕ ").monospace().color(line_colour));
                        match &data.info_artist {
                            Some(x) => {
                                let y = match x.len() > 32 {
                                    true => {
                                        // this dumb shit is currently required because truncate panics too often
                                        let mut z = String::new();
                                        for (i, c) in x.chars().enumerate() {
                                            z.push(c);
                                            if i == 29 {
                                                break
                                            }
                                        }
                                        z.push(' ');
                                        z.push('…');
                                        z
                                    }
                                    false => x.to_owned()
                                };
                                ui.label(RichText::new(y).monospace().color(data.colours.base04))
                            }
                            None => ui.label(RichText::new("unknown!").monospace().color(data.colours.base02))
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label(RichText::new(" ").monospace().color(data.colours.base04));
                        ui.label(RichText::new("▕ ").monospace().color(line_colour));
                        match &data.info_album {
                            Some(x) => {
                                let y = match x.len() > 32 {
                                    true => {
                                        // this dumb shit is currently required because truncate panics too often
                                        let mut z = String::new();
                                        for (i, c) in x.chars().enumerate() {
                                            z.push(c);
                                            if i == 29 {
                                                break
                                            }
                                        }
                                        z.push(' ');
                                        z.push('…');
                                        z
                                    }
                                    false => x.to_owned()
                                };
                                ui.label(RichText::new(y).monospace().color(data.colours.base04))
                            }
                            None => ui.label(RichText::new("unknown!").monospace().color(data.colours.base02))
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label(RichText::new(" ").monospace().color(data.colours.base04));
                        ui.label(RichText::new("▕ ").monospace().color(line_colour));
                        match &data.info_duration {
                            Some(x) => ui.label(RichText::new(x).monospace().color(data.colours.base04)),
                            None => ui.label(RichText::new("unknown!").monospace().color(data.colours.base02))
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label(RichText::new(" ").monospace().color(data.colours.base04));
                        ui.label(RichText::new("▕ ").monospace().color(line_colour));
                        match &data.info_date {
                            Some(x) => ui.label(RichText::new(x).monospace().color(data.colours.base04)),
                            None => ui.label(RichText::new("unknown!").monospace().color(data.colours.base02))
                        }
                    });

                    ui.add_space(20.0);
                    let filler = ui.available_height() - 80.0;
                    if filler > 0.0 {
                        ui.add_space(filler)
                    }
                });

                let elapsed = data.elapsed.unwrap_or(0);
                let duration = data.duration.unwrap_or(0);
                let progress_label = utils::progress_string(elapsed, duration);
                let mut seek_pos = elapsed as f32;

                let (state_icon, progress_colour, state_colour, slider_colour) = match data.state {
                    State::Pause => ("", data.colours.base03, data.colours.base0F, data.colours.base0F),
                    State::Play => ("", data.colours.base04, data.colours.base09, data.colours.base09),
                    State::Stop => ("", data.colours.base01, data.colours.base01, data.colours.base01)
                };

                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(progress_label).small().color(progress_colour))
                });
                ui.add_space(3.0);
                ui.horizontal(|ui| {
                    ui.add_space(20.0);
                    ui.label(RichText::new(state_icon).heading().color(state_colour));
                    ui.add_space(36.0);
                    if data.state != State::Stop {
                        ui.visuals_mut().widgets.active.bg_fill = data.colours.base01;
                        ui.visuals_mut().widgets.hovered.bg_fill = data.colours.base01;
                        ui.visuals_mut().widgets.inactive.bg_fill = data.colours.base01;
                        ui.visuals_mut().widgets.inactive.fg_stroke = Stroke {
                            width: 1.2,
                            color: slider_colour
                        };
                        ui.visuals_mut().widgets.active.fg_stroke = Stroke {
                            width: 1.6,
                            color: data.colours.base0C
                        };
                        ui.visuals_mut().widgets.hovered.fg_stroke = Stroke {
                            width: 1.6,
                            color: data.colours.base0C
                        };
                        let seek =
                            ui.add(Slider::new(&mut seek_pos, 0.0..=duration as f32).show_value(false));
                        if seek.clicked() || seek.drag_released() {
                            data.mpc.rewind((seek_pos / 1000.0).floor() as i64).unwrap()
                        }
                    }
                    else {
                        ui.visuals_mut().widgets.noninteractive.bg_fill = data.colours.base00;
                        ui.visuals_mut().widgets.noninteractive.fg_stroke = Stroke {
                            width: 1.2,
                            color: data.colours.base03
                        };
                        let _seek = ui.add_enabled(
                            false,
                            Slider::new(&mut seek_pos, 0.0..=duration as f32).show_value(false)
                        );
                    }
                });

                ui.vertical(|ui| {
                    ui.add_space(3.0);
                    if data.switcher_cycle == 3 {
                        ui.label(RichText::new(&data.switcher).small().color(data.colours.base03))
                    }
                    else {
                        match data.switcher.len() > 47 {
                            true => {
                                // this dumb shit is currently required because truncate panics too often
                                let mut chop = String::new();
                                for (i, c) in data.switcher.chars().enumerate() {
                                    chop.push(c);
                                    if i == 45 {
                                        break
                                    }
                                }
                                chop.push(' ');
                                chop.push('…');
                                ui.label(RichText::new(&chop).small().color(data.colours.base04))
                            }
                            false => {
                                ui.label(RichText::new(&data.switcher).small().color(data.colours.base04))
                            }
                        }
                    }
                });

                ui.add_space(2.0)
            });

        CentralPanel::default()
            .frame(eframe::egui::containers::Frame {
                margin: Vec2::new(8.0, 8.0),
                corner_radius: 0.0,
                fill: data.colours.base00,
                stroke: Default::default(),
                ..Default::default()
            })
            .show(ctx, |ui| {
                ui.visuals_mut().dark_mode = true;
                ui.visuals_mut().extreme_bg_color = data.colours.base00;
                ui.visuals_mut().widgets.noninteractive.bg_stroke.color = data.colours.base01;
                ui.visuals_mut().widgets.active.bg_fill = data.colours.base0C;
                ui.visuals_mut().widgets.hovered.bg_fill = data.colours.base0C;
                ui.visuals_mut().widgets.inactive.bg_fill = data.colours.base01;
                ui.group(|ui| {
                    match data.list.is_empty() {
                        true => {
                            ui.vertical_centered_justified(|ui| {
                                ui.add_space(30.0);
                                ui.label(
                                    RichText::new("¯\\_(ツ)_/¯")
                                        .text_style(TextStyle::Button)
                                        .color(data.colours.base08)
                                );
                                ui.add_space(ui.available_height() - 42.0)
                            });
                            data.selected = 0;
                            data.selected_pos = data.current_pos
                        }
                        false => {
                            ScrollArea::vertical()
                                .auto_shrink([false; 2])
                                .max_height(ui.available_height() - 42.0)
                                .show(ui, |ui| {
                                    ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                                        let selected_bg = match data.selected == data.current_pos {
                                            true => data.colours.base09,
                                            false => data.colours.base0F
                                        };
                                        ui.visuals_mut().selection = Selection {
                                            bg_fill: selected_bg,
                                            stroke:  Stroke {
                                                width: 0.0,
                                                color: data.colours.base00
                                            }
                                        };
                                        ui.visuals_mut().widgets.active.bg_fill = data.colours.base01;
                                        ui.visuals_mut().widgets.hovered.bg_fill = data.colours.base01;
                                        ui.visuals_mut().widgets.active.bg_stroke = Stroke {
                                            width: 1.2,
                                            color: data.colours.base09
                                        };
                                        ui.visuals_mut().widgets.hovered.bg_stroke = Stroke {
                                            width: 1.0,
                                            color: data.colours.base0F
                                        };
                                        for (i, song) in data.list.iter().enumerate() {
                                            let text_colour = match i == data.selected {
                                                true => data.colours.base00,
                                                false if i == data.current_pos => data.colours.base09,
                                                false => data.colours.base04
                                            };
                                            let entry = ui.selectable_label(
                                                i == data.selected,
                                                RichText::new(&song.title).monospace().color(text_colour)
                                            );
                                            if i == data.selected {
                                                data.selected_pos = song.pos;
                                                if data.need_list_scroll {
                                                    entry.scroll_to_me(Align::Center);
                                                    if data.interacted {
                                                        data.need_list_scroll = false
                                                    }
                                                }
                                            }
                                            if entry.clicked() {
                                                data.selected = i;
                                                data.selected_pos = song.pos;
                                                data.interacted = true
                                            }
                                            if entry.double_clicked()
                                                && data.mpc.switch(song.pos as u32).is_ok()
                                            {
                                                frame.quit()
                                            }
                                        }
                                    })
                                });
                        }
                    }
                    ui.separator();
                    ui.add_space(2.0);

                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(0.0, 0.0);
                        ui.add_space(4.0);
                        ui.label(RichText::new("").heading().color(data.colours.base0F));
                        ui.add_space(10.0);
                        ui.vertical_centered_justified(|ui| {
                            ui.set_max_width(330.0);
                            ui.visuals_mut().selection = Selection {
                                bg_fill: data.colours.base0F,
                                stroke:  Stroke {
                                    width: 1.2,
                                    color: data.colours.base09
                                }
                            };

                            let search = ui.add(
                                TextEdit::singleline(&mut data.search_query)
                                    .frame(false)
                                    .text_color(data.colours.base05)
                            );
                            search.request_focus();

                            if search.changed() {
                                data.list = search::build_list(&data.search_query, &data.queue);
                                if !data.search_query.is_empty() {
                                    data.selected = 0;
                                    data.interacted = true
                                }
                                else {
                                    data.selected = data.current_pos;
                                    data.need_list_scroll = true;
                                    data.interacted = false
                                }
                            }
                        })
                    });
                })
            });

        frame.request_repaint()
    }
}
