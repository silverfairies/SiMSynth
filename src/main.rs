#![allow(unused)]

use rodio::Source;
use rodio::mixer::Mixer;
use rodio::source::SineWave;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::{thread, vec};

fn main() {
    let mut stream_handle = rodio::DeviceSinkBuilder::from_default_device()
        .unwrap()
        .open_stream()
        .unwrap();
    stream_handle.log_on_drop(false);
    let mixer = stream_handle.mixer();

    let beep0 = Sound::new(440.0, mixer);
    let beep1 = Sound::new(523.25, mixer);
    let beep2 = Sound::new(659.26, mixer);

    beep0.play();
    beep1.play();
    beep2.play();

    thread::sleep(Duration::from_millis(1500));

    beep1.pause();
    beep2.pause();

    thread::sleep(Duration::from_millis(1500));

    beep0.pause();
    beep1.play();

    thread::sleep(Duration::from_millis(1500));

    beep1.pause();
    beep2.play();

    thread::sleep(Duration::from_millis(1500));

    beep0.play();
    beep1.play();
    beep2.drop();

    thread::sleep(Duration::from_millis(1500));
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
    scale: Vec<f32>,
}

#[derive(Debug)]
struct Button {}

struct Sound {
    frequency: f32,
    paused: Arc<AtomicBool>,
    dropped: Arc<AtomicBool>,
}

impl Sound {
    fn new(frequency: f32, mixer: &Mixer) -> Sound {
        let pause_sound = Arc::new(AtomicBool::new(true));
        let clone_pause_sound = pause_sound.clone();
        let dropped = Arc::new(AtomicBool::new(false));
        let clone_dropped = dropped.clone();
        let mut wave = SineWave::new(frequency)
            .amplify(0.2)
            .pausable(true)
            .skippable()
            .periodic_access(Duration::from_millis(200), move |wave| {
                if clone_dropped.load(Ordering::Relaxed) {
                    wave.skip();
                } else if !wave.inner().is_paused() && clone_pause_sound.load(Ordering::Relaxed) {
                    wave.inner_mut().set_paused(true);
                } else if wave.inner().is_paused() && !clone_pause_sound.load(Ordering::Relaxed) {
                    wave.inner_mut().set_paused(false);
                }
            });
        mixer.add(wave);
        Sound {
            frequency,
            paused: pause_sound,
            dropped,
        }
    }

    fn pause(&self) {
        self.paused.store(true, Ordering::Relaxed);
    }

    fn play(&self) {
        self.paused.store(false, Ordering::Relaxed);
    }

    fn drop(self) {
        self.dropped.store(true, Ordering::Relaxed);
    }
}
