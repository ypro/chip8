mod arch;
mod ram;
mod regs;
mod chip;
mod instr;
mod framebuffer;
mod ui;
mod util;
mod profile;

use std::io::Write;
use std::io::Read;
use std::fs::File;
use std::env;
use std::thread::sleep;
use std::time::Duration;

use log::{info, trace};

use crate::ui::Event;
use crate::profile::Profile;

fn main() -> std::io::Result<()>{

    env_logger::init();

    let args = clap::App::new("Chip-8 emulator")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .arg(clap::Arg::new("rom_path")
             .help("ROM file name.")
             .long("rom_path")
             .short('r')
             .value_name("path")
             .takes_value(true)
             .default_value("rom/tests/ibm.ch8"))
        .arg(clap::Arg::new("profile")
             .help("Chip-8 profile.")
             .long("profile")
             .short('p')
             .value_parser(["original", "modern"])
             .default_value("modern"))
        .arg(clap::Arg::new("fast")
             .help("Run emulation as fast as possible.")
             .long("fast")
             .short('f')
             .action(clap::ArgAction::SetTrue))
        .get_matches();

    let rom_name = args.get_one::<String>("rom_path").unwrap();
    let mut f = File::open(rom_name)?;

    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;

    let profile = match args.get_one::<String>("profile").unwrap().as_str() {
        "original" => Profile::original(),
        "modern" => Profile::modern(),
        _ => unreachable!(),
    };

    let fast = args.get_one::<bool>("fast").unwrap();

    let mut chip = chip::Chip::new(profile);

    chip.load_rom(&buffer, 0x200);
    chip.set_pc(0x200);

    let mut ui = ui::Ui::new();

    let mut running = true;

    let start_ms = ui.timers.get_ms();
    let mut cycles: u64 = 0;
    let mut last_frame_ms = start_ms;
    let frame_interval: [u32; 3] = [17, 17, 16];
    let mut frame_idx = 0;

    let mut no_frame_cycles: u64 = 0;

    while running {
        let now_ms = ui.timers.get_ms();
        let frame_sync = now_ms - last_frame_ms > frame_interval[frame_idx];

        if frame_sync {
            last_frame_ms = now_ms;
            frame_idx += 1;
            if frame_idx == frame_interval.len() {
                frame_idx = 0;
            }

            for e in ui.events.poll_iter() {
                match e {
                    Event::Quit =>  { info!("Quit!"); std::io::stdout().flush().unwrap(); running = false },
                    Event::KeyPress(key) => { trace!("Key pressed: {}", key); chip.key_press(key) },
                    Event::KeyUnpress(key) => { trace!("Key unpressed {}", key); chip.key_unpress(key) },
                }
            }
        }

        if running {
            cycles += 1;

            if frame_sync {
                info!("frame_sync");
                chip.cycle_timers();
                if chip.is_sound_on() {
                    ui.audio.on();
                } else {
                    ui.audio.off();
                }
            } else {
                no_frame_cycles += 1;
            }
            chip.cycle();

            if frame_sync {
                let f: &framebuffer::Frame = chip.get_frame();
                ui.display.present_frame(f);
            }
        }
        if !fast {
            sleep(Duration::from_millis(1));
        }
    }
    let end_ms = ui.timers.get_ms();
    let duration_ms = end_ms - start_ms;
    let cps: f64 = 1000.0 * cycles as f64 / duration_ms as f64;

    println!("Stats.");
    println!("Execution time: {} ms", duration_ms);
    println!("Cycles: {}", cycles);
    println!("Cycles per second: {}", cps);
    println!("No frame cycles: {}", no_frame_cycles);

    Ok(())
}
