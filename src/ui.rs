extern crate sdl2;

use log::trace;

use sdl2::keyboard::Keycode;
use sdl2::{pixels::Color, rect::Rect};

use crate::arch;
use crate::framebuffer::Frame;

const PIXEL_SIZE: u32 = 14;
const BORDER_SIZE: u32 = 1;
const PIXEL_INNER_SIZE: u32 = PIXEL_SIZE - 2 * BORDER_SIZE;

const SCREEN_WIDTH: u32 = PIXEL_SIZE * arch::DISPLAY_WIDTH;
const SCREEN_HEIGHT: u32 = PIXEL_SIZE * arch::DISPLAY_HEIGHT;

const BACKGROUND_COLOR: Color = Color::BLUE;
const PIXEL_COLOR: Color = Color::RGB(200, 200, 200);

pub enum Event {
    KeyPress(u8),
    KeyUnpress(u8),
    Quit,
}

pub struct EventIterator<'a> {
    event_pump: &'a mut sdl2::EventPump,
}

impl<'a> Iterator for EventIterator<'a> {
    type Item = Event;

    fn next(self: &mut EventIterator<'a>) -> Option<Self::Item> {
        Events::match_event(self.event_pump.poll_event())
    }
}

pub struct Display {
    canvas: sdl2::render::WindowCanvas,
}

impl Display {
    pub fn new(canvas: sdl2::render::WindowCanvas) -> Display {
        Display {
            canvas,
        }
    }

    pub fn present_frame(&mut self, frame: &Frame) {
        self.canvas.set_draw_color(BACKGROUND_COLOR);
        self.canvas.clear();
        self.canvas.set_draw_color(PIXEL_COLOR);
        let mut pixels: Vec<Rect> = Vec::new();
        for (i, row) in frame.iter().enumerate() {
            for (j, p) in row.iter().enumerate() {
                if *p!= 0 {
                    let x: i32 = (PIXEL_SIZE * (j as u32) + BORDER_SIZE) as i32;
                    let y: i32 = (PIXEL_SIZE * (i as u32) + BORDER_SIZE) as i32;

                    pixels.push(Rect::new(x, y, PIXEL_INNER_SIZE, PIXEL_INNER_SIZE));
                    //println!("Draw pixel {} {} {} {}", x,y, PIXEL_INNER_SIZE, PIXEL_INNER_SIZE);
                }
            }
        }
        self.canvas.fill_rects(&pixels).unwrap();
        self.canvas.present();
    }
}

pub struct Events {
    event_pump: sdl2::EventPump,
}

impl Events {
    pub fn new(event_pump: sdl2::EventPump) -> Events {
        Events {
            event_pump,
        }
    }

    pub fn poll_iter(&mut self) -> EventIterator {
        EventIterator {
            event_pump: &mut self.event_pump,
        }
    }

    fn match_event(sdl2_event: Option<sdl2::event::Event>) -> Option<Event> {
         match sdl2_event {
            Some(sdl2::event::Event::Quit {..}) |
                Some(sdl2::event::Event::KeyDown { keycode: Some(Keycode::Space), repeat: false, .. }) => Some(Event::Quit),

            // Row 1
            Some(sdl2::event::Event::KeyDown { keycode: Some(Keycode::Num1), repeat: false, .. }) => Some(Event::KeyPress(0x1)),
            Some(sdl2::event::Event::KeyUp { keycode: Some(Keycode::Num1), repeat: false, .. }) => Some(Event::KeyUnpress(0x1)),

            Some(sdl2::event::Event::KeyDown { keycode: Some(Keycode::Num2), repeat: false, .. }) => Some(Event::KeyPress(0x2)),
            Some(sdl2::event::Event::KeyUp { keycode: Some(Keycode::Num2), repeat: false, .. }) => Some(Event::KeyUnpress(0x2)),

            Some(sdl2::event::Event::KeyDown { keycode: Some(Keycode::Num3), repeat: false, .. }) => Some(Event::KeyPress(0x3)),
            Some(sdl2::event::Event::KeyUp { keycode: Some(Keycode::Num3), repeat: false, .. }) => Some(Event::KeyUnpress(0x3)),

            Some(sdl2::event::Event::KeyDown { keycode: Some(Keycode::Num4), repeat: false, .. }) => Some(Event::KeyPress(0xC)),
            Some(sdl2::event::Event::KeyUp { keycode: Some(Keycode::Num4), repeat: false, .. }) => Some(Event::KeyUnpress(0xC)),

            // Row 2
            Some(sdl2::event::Event::KeyDown { keycode: Some(Keycode::Q), repeat: false, .. }) => Some(Event::KeyPress(0x4)),
            Some(sdl2::event::Event::KeyUp { keycode: Some(Keycode::Q), repeat: false, .. }) => Some(Event::KeyUnpress(0x4)),

            Some(sdl2::event::Event::KeyDown { keycode: Some(Keycode::W), repeat: false, .. }) => Some(Event::KeyPress(0x5)),
            Some(sdl2::event::Event::KeyUp { keycode: Some(Keycode::W), repeat: false, .. }) => Some(Event::KeyUnpress(0x5)),

            Some(sdl2::event::Event::KeyDown { keycode: Some(Keycode::E), repeat: false, .. }) => Some(Event::KeyPress(0x6)),
            Some(sdl2::event::Event::KeyUp { keycode: Some(Keycode::E), repeat: false, .. }) => Some(Event::KeyUnpress(0x6)),

            Some(sdl2::event::Event::KeyDown { keycode: Some(Keycode::R), repeat: false, .. }) => Some(Event::KeyPress(0xD)),
            Some(sdl2::event::Event::KeyUp { keycode: Some(Keycode::R), repeat: false, .. }) => Some(Event::KeyUnpress(0xD)),

            // Row 3
            Some(sdl2::event::Event::KeyDown { keycode: Some(Keycode::A), repeat: false, .. }) => Some(Event::KeyPress(0x7)),
            Some(sdl2::event::Event::KeyUp { keycode: Some(Keycode::A), repeat: false, .. }) => Some(Event::KeyUnpress(0x7)),

            Some(sdl2::event::Event::KeyDown { keycode: Some(Keycode::S), repeat: false, .. }) => Some(Event::KeyPress(0x8)),
            Some(sdl2::event::Event::KeyUp { keycode: Some(Keycode::S), repeat: false, .. }) => Some(Event::KeyUnpress(0x8)),

            Some(sdl2::event::Event::KeyDown { keycode: Some(Keycode::D), repeat: false, .. }) => Some(Event::KeyPress(0x9)),
            Some(sdl2::event::Event::KeyUp { keycode: Some(Keycode::D), repeat: false, .. }) => Some(Event::KeyUnpress(0x9)),

            Some(sdl2::event::Event::KeyDown { keycode: Some(Keycode::F), repeat: false, .. }) => Some(Event::KeyPress(0xE)),
            Some(sdl2::event::Event::KeyUp { keycode: Some(Keycode::F), repeat: false, .. }) => Some(Event::KeyUnpress(0xE)),

            // Row 4
            Some(sdl2::event::Event::KeyDown { keycode: Some(Keycode::Z), repeat: false, .. }) => Some(Event::KeyPress(0xA)),
            Some(sdl2::event::Event::KeyUp { keycode: Some(Keycode::Z), repeat: false, .. }) => Some(Event::KeyUnpress(0xA)),

            Some(sdl2::event::Event::KeyDown { keycode: Some(Keycode::X), repeat: false, .. }) => Some(Event::KeyPress(0x0)),
            Some(sdl2::event::Event::KeyUp { keycode: Some(Keycode::X), repeat: false, .. }) => Some(Event::KeyUnpress(0x0)),

            Some(sdl2::event::Event::KeyDown { keycode: Some(Keycode::C), repeat: false, .. }) => Some(Event::KeyPress(0xB)),
            Some(sdl2::event::Event::KeyUp { keycode: Some(Keycode::C), repeat: false, .. }) => Some(Event::KeyUnpress(0xB)),

            Some(sdl2::event::Event::KeyDown { keycode: Some(Keycode::V), repeat: false, .. }) => Some(Event::KeyPress(0xF)),
            Some(sdl2::event::Event::KeyUp { keycode: Some(Keycode::V), repeat: false, .. }) => Some(Event::KeyUnpress(0xF)),

            _ => None,
        }
    }
}

pub struct Timers {
    pub timer_subsystem: sdl2::TimerSubsystem,
}

impl Timers {
    pub fn new(timer_subsystem: sdl2::TimerSubsystem) -> Timers {
        Timers {
            timer_subsystem,
        }
    }

    pub fn get_ms(&self) -> u32 {
        self.timer_subsystem.ticks()
    }
}

pub struct Audio {
    dev: sdl2::audio::AudioDevice<SinWave>,
    is_on: bool,
}

struct SinWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl SinWave {
    pub fn new(freq: f32, spec: &sdl2::audio::AudioSpec) -> SinWave {
        SinWave {
                phase_inc: freq / spec.freq as f32,
                phase: 0.0,
                volume: 0.25,
        }
    }
}

impl sdl2::audio::AudioCallback for SinWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        for i in out.iter_mut() {
            let phase = self.phase * 2.0 * std::f32::consts::PI;
            *i = phase.sin() * self.volume;

            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

impl Audio {
    pub fn new(audio_subsystem: sdl2::AudioSubsystem) -> Audio {
        let spec = sdl2::audio::AudioSpecDesired {
            freq: Some(44100),
            channels: Some(1),
            samples: None,
        };
        let dev = audio_subsystem.open_playback(None, &spec, |spec| {
            SinWave::new(440.0, &spec)
        }).unwrap();
        Audio {
            dev,
            is_on: false,
        }
    }

    pub fn on(&mut self) {
        if self.is_on {
            return;
        }
        trace!("Sound on");
        self.dev.resume();
        self.is_on = true;
    }

    pub fn off(&mut self) {
        if !self.is_on {
            return;
        }
        trace!("Sound off");
        self.dev.pause();
        self.is_on = false;
    }
}

pub struct Ui {
    pub display: Display,
    pub events: Events,
    pub timers: Timers,
    pub audio: Audio,
}

impl Ui {
    pub fn new() -> Self {
        let sdl_ctx = sdl2::init().unwrap();
        let video = sdl_ctx.video().unwrap();
        let window = video.window("rust-sdl2 demo", SCREEN_WIDTH, SCREEN_HEIGHT)
            .position_centered()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().accelerated().build().unwrap();
        canvas.set_draw_color(BACKGROUND_COLOR);
        canvas.clear();
        canvas.present();

        let event_pump = sdl_ctx.event_pump().unwrap();
        let timer_subsystem = sdl_ctx.timer().unwrap();
        let audio_subsystem = sdl_ctx.audio().unwrap();

        Ui {
            display: Display::new(canvas),
            events: Events::new(event_pump),
            timers: Timers::new(timer_subsystem),
            audio: Audio::new(audio_subsystem),
        }
    }
}
