#![allow(unused)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use egui::WidgetText::RichText;
use rodio::mixer::Mixer;
use rodio::source::SineWave;
use rodio::{MixerDeviceSink, Source};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::{thread, vec};

fn main() -> eframe::Result {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };

    /*
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
    */
    eframe::run_native(
        "SimpSynth",
        native_options,
        Box::new(|cc| Ok(Box::new(SiSApp::new(cc)))),
    )
}

struct SiSApp {
    env: Environment,
    scales: Vec<SoundMap>,
}

impl Default for SiSApp {
    fn default() -> Self {
        Self {
            env: Environment::default(),
            scales: vec![SIMPLE3SCALE, STANDART8SCALE],
        }
    }
}

impl SiSApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_global_style.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

impl eframe::App for SiSApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("Hello World!");
            ui.horizontal_top(|ui| {
                for sound in &self.env.buttons {
                    if ui
                        .toggle_value(&mut false, sound.frequency.to_string())
                        .clicked()
                    {
                        sound.toggle();
                    }
                }
            });
            if ui.button("init").clicked() {
                self.env.init();
            }
            egui::ComboBox::from_label("Scale")
                .selected_text(self.env.scale.name)
                .show_ui(ui, |ui| {
                    for scale in &self.scales {
                        if ui.selectable_label(false, scale.name).clicked() {
                            self.env.scale = scale.clone();
                            self.env.init();
                        }
                    }
                })
        });
    }
}

struct Environment {
    scale: SoundMap,
    recording: bool,
    buttons: Vec<Sound>,
    sink: MixerDeviceSink,
    mixer: Mixer,
}

impl Environment {
    fn new(scale: SoundMap) -> Environment {
        let mut stream_handle = rodio::DeviceSinkBuilder::from_default_device()
            .unwrap()
            .open_stream()
            .unwrap();
        //stream_handle.log_on_drop(false);
        let mixer = stream_handle.mixer();

        let mut env = Environment {
            scale,
            recording: false,
            buttons: vec![],
            mixer: mixer.to_owned(),
            sink: stream_handle,
        };
        env.init();
        env
    }

    fn init(&mut self) {
        for sound in &self.buttons {
            sound.drop();
        }

        self.buttons = Vec::new();

        for frequency in self.scale.scale {
            self.buttons
                .push(Sound::new(frequency.clone(), &self.mixer));
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new(SIMPLE3SCALE)
    }
}

#[derive(Clone, Copy)]
pub struct SoundMap {
    name: &'static str,
    scale: &'static [f32],
}

impl SoundMap {
    const fn new(name: &'static str, scale: &'static [f32]) -> Self {
        Self { name, scale }
    }
}

pub struct Button {}

pub struct Sound {
    pub frequency: f32,
    paused: Arc<AtomicBool>,
    dropped: Arc<AtomicBool>,
}

impl Sound {
    pub fn new(frequency: f32, mixer: &Mixer) -> Self {
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
        Self {
            frequency,
            paused: pause_sound,
            dropped,
        }
    }

    pub fn pause(&self) {
        self.change_state(true);
    }

    pub fn play(&self) {
        self.change_state(false);
    }

    pub fn drop(&self) {
        self.dropped.store(true, Ordering::Relaxed);
    }

    pub fn toggle(&self) {
        if self.paused.load(Ordering::Relaxed) {
            self.play();
        } else {
            self.pause();
        }
    }

    fn change_state(&self, pause: bool) {
        self.paused.store(pause, Ordering::Relaxed);
    }
}

const STANDART8SCALE: SoundMap = SoundMap::new(
    "Western A scale",
    &[
        440.00, 493.88, 523.25, 587.33, 659.26, 698.46, 783.99, 880.00,
    ],
);

const SIMPLE3SCALE: SoundMap = SoundMap::new("Simple test scale", &[440.00, 587.33]);
