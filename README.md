# rinse
#### a fast song selector for mpd

The way I typically use mpd is to add all of my songs to a single playlist with random enabled and use mpc with mediakey keyboard shortcuts to control playback.  
This works well enough until there is a specific song that I want to listen to, at which point I need to use some other app to find the song.  

I have used a few different things in the past to achieve that (rofi + a shell script, ncmpcpp) but none really worked or looked quite how I wanted, so I created this.

Built using the fantastic [egui](https://github.com/emilk/egui) library, this is designed to do this singular task as well as possible (at least to the current limit of my skills)


https://user-images.githubusercontent.com/53883649/153692419-f91dc91b-52a9-486e-b953-8a74a70fb7b6.mp4

##### This is still very much alpha quality, not sure how well it's going to work on other people's setup, feedback greatly appreciated!

### Install instructions

#### deps

mpd (obviously)  
rust/cargo  

---

clone repo and cd into repo root  
run ```cargo build --release```  
then ```sudo cp ./target/release/rinse /usr/local/bin```

it is recommended to setup a dedicated keybind in your DE/WM to launch rinse

### keys

**tab / shift+tab** - scroll down/up  
**ctrl+u** - clear search and highlight current song  
**enter** - play selected song  
**esc** - exit

---

#### fonts used

[Iosevka](https://github.com/be5invis/Iosevka)  
[Victor Mono Italic](https://github.com/rubjo/victor-mono)  

#### colour scheme

by default this uses the [nord base16](https://github.com/ada-lovecraft/base16-nord-scheme) colour scheme.  
if you wish to change the colours you can [download](https://github.com/chriskempson/base16) any base16 scheme and then copy it to ```$XDG_CONFIG_HOME/rinse/theme.yaml```

---
#### install script, updates, etc coming soon!
