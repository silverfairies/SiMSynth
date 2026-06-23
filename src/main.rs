#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;
use egui_winit::winit;
use egui_winit::winit::application::ApplicationHandler;
use egui_winit::winit::platform::scancode::PhysicalKeyExtScancode;
use rodio::mixer::Mixer;
use rodio::source::SineWave;
use rodio::{MixerDeviceSink, Source};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::vec;

const VERSION: &str = "0.2.0-alpha";

fn main() -> eframe::Result {
    env_logger::init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0]),
        ..Default::default()
    };

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

impl SiSApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_global_style.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        //let wtest = cc.display_handle().unwrap();
        //let rtest = wtest.as_raw();
        cc.egui_ctx.enable_accesskit();
        Self {
            env: Environment::default(),
            scales: vec![SIMPLE2A4SCALE0, WESTERN8A4SCALE, WESTERN8C4SCALE],
        }
    }
}

#[allow(unused)]
impl ApplicationHandler for SiSApp {
    // TODO: Code to try to obtain keycodes. This implementation DOES NOT YET WORK
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if let winit::event::WindowEvent::KeyboardInput {
            device_id,
            event,
            is_synthetic,
        } = event
        {
            println!("{}", event.physical_key.to_scancode().unwrap());
        }
    }

    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {}
}

impl eframe::App for SiSApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading(format!(
                "Simple Music Synthetizer: {}",
                VERSION //std::env::var("CARGO_PKG_VERSION").unwrap()
            ));
            ui.horizontal_top(|ui| {
                //Dynamic generation of Buttons for each sound
                for sound in &self.env.buttons {
                    if ui
                        .toggle_value(
                            &mut sound.paused.load(Ordering::Relaxed),
                            sound.frequency.to_string(),
                        )
                        .clicked()
                    {
                        sound.toggle();
                    }
                }
            });
            if ui.button("reload").clicked() {
                self.env.reload();
            }

            // Attempt at a keyboard handler. Only works for the fers two buttons yet, a and s keys correspondigly
            // TODO: reimplement with a dynamic generation
            //let wtest = frame.winit_window().unwrap();
            let input = ui.input(|i| i.events.clone());
            if !input.is_empty() {
                //println!("Something happened: {:?}", input);
                for ievent in input {
                    if let egui::Event::Key {
                        key: _,
                        physical_key,
                        pressed: _,
                        repeat: false,
                        modifiers: _,
                    } = ievent
                    {
                        match physical_key.unwrap() {
                            egui::Key::A => {
                                println!("Something happened!");
                                self.env.buttons[0].toggle();
                            }
                            egui::Key::S => {
                                self.env.buttons[1].toggle();
                            }
                            _ => (),
                        }
                    }
                }
            }

            egui::ComboBox::from_label("Scale") //Dropdown menu to choose the set of sounds
                .selected_text(self.env.scale.name)
                .show_ui(ui, |ui| {
                    for scale in &self.scales {
                        if ui.selectable_label(false, scale.name).clicked() {
                            self.env.scale = scale.to_owned();
                            self.env.reload();
                        }
                    }
                })
        });
    }
}

struct Environment {
    //here are is all the relevant information for the curent session stored
    scale: SoundMap,
    //recording: bool,
    buttons: Vec<Sound>,
    _sink: MixerDeviceSink,
    mixer: Mixer,
}

impl Environment {
    //Gets a MixerDeviceSink and a Mixer to put sound information into
    fn new(scale: SoundMap) -> Environment {
        let stream_handle = rodio::DeviceSinkBuilder::from_default_device()
            .unwrap()
            .open_stream()
            .unwrap();
        //stream_handle.log_on_drop(false);
        let mixer = stream_handle.mixer();

        let mut env = Environment {
            scale,
            //recording: false,
            buttons: vec![],
            mixer: mixer.to_owned(),
            _sink: stream_handle,
        };
        env.reload();
        env
    }

    //Regeneration of buttons
    fn reload(&mut self) {
        for sound in &self.buttons {
            sound.drop();
        }

        self.buttons = Vec::new();

        for frequency in self.scale.scale {
            self.buttons
                .push(Sound::new(frequency.to_owned(), &self.mixer));
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new(SIMPLE2A4SCALE0)
    }
}

//Struct for scales
#[derive(Clone, Copy)]
pub struct SoundMap {
    name: &'static str,
    scale: &'static [f32],
    //row: u32,
}

impl SoundMap {
    const fn new(name: &'static str, scale: &'static [f32], _row: u32) -> Self {
        Self {
            name,
            scale, /*, row */
        }
    }
}

//Access to sounds given to the Mixer. Paused and dropped are used to control the corresponding sound. Frequency is saved for naming purposes.
pub struct Sound {
    pub frequency: f32,
    paused: Arc<AtomicBool>,
    dropped: Arc<AtomicBool>,
}

//Creates a sinewave, makes it pausable and skippable, with a periodic_access to pause/resume/drop. Passes the vawe to the given Mixer, innitialy paused, so it doesn't play when not needed.
impl Sound {
    pub fn new(frequency: f32, mixer: &Mixer) -> Self {
        let paused = Arc::new(AtomicBool::new(true));
        let clone_pause_sound = paused.clone();
        let dropped = Arc::new(AtomicBool::new(false));
        let clone_dropped = dropped.clone();
        let wave = SineWave::new(frequency)
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
            paused,
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

    fn change_state(&self, pause: bool) -> bool {
        self.paused.store(pause, Ordering::Relaxed);
        pause
    }
}

//Arrays for the scales
//TODO: read from file
const WESTERN8A4SCALE: SoundMap = SoundMap::new(
    "Western A Octave",
    &[
        440.00, 493.88, 523.25, 587.33, 659.26, 698.46, 783.99, 880.00,
    ],
    7,
);

const SIMPLE2A4SCALE0: SoundMap = SoundMap::new("Simple test", &[440.00, 587.33], 2);

const WESTERN8C4SCALE: SoundMap = SoundMap::new(
    "Western C Octave",
    &[
        261.63, 293.67, 329.63, 349.23, 392.00, 440.00, 493.88, 523.25,
    ],
    7,
);
