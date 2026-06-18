#![allow(unused)]

use rodio::Source;
use rodio::source::SineWave;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

fn main() {
    let sc = SoundMap {
        name: String::from("test"),
    };
    let env = Environment::new(&sc);
    println!("{:#?}", env);
    let stream_handle = rodio::DeviceSinkBuilder::from_default_device()
        .unwrap()
        .open_stream()
        .unwrap();
    let mixer = stream_handle.mixer();

    let stop_first_beep = Arc::new(AtomicBool::new(false));
    let clone_stop_first_beep = stop_first_beep.clone();
    let mut wave = SineWave::new(740.0)
        .amplify(0.2)
        .repeat_infinite()
        .stoppable()
        .periodic_access(Duration::from_millis(200), move |wave| {
            if clone_stop_first_beep.load(Ordering::Relaxed) {
                wave.stop();
            }
        });
    mixer.add(wave);
    println!("Started beep2");
    thread::sleep(Duration::from_millis(1500));

    {
        // Generate sine wave.
        let wave = SineWave::new(440.0)
            .amplify(0.2)
            .take_duration(Duration::from_secs(3));
        mixer.add(wave);
    }
    println!("Started beep");
    stop_first_beep.store(true, Ordering::Relaxed);
    thread::sleep(Duration::from_millis(3000));
}

#[derive(Debug)]
struct Environment<'s> {
    scale: &'s SoundMap,
    recording: bool,
    buttons: Vec<Button>,
}

impl Environment<'_> {
    fn new(scale: &SoundMap) -> Environment<'_> {
        Environment {
            scale,
            recording: false,
            buttons: vec![Button {}, Button {}],
        }
    }
}

#[derive(Debug)]
struct SoundMap {
    name: String,
}

#[derive(Debug)]
struct Button {}
