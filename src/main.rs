//! anti-afk - randomized input keep-alive for Roblox on Windows.
//!
//! Taps a key every so often while the Roblox window is focused, so the
//! server's inactivity timer never fires. F8 pauses, F9 quits.

mod config;
mod input;
mod keys;
mod rng;
mod window;

use std::thread::sleep;
use std::time::{Duration, Instant};

use config::Config;
use input::{EdgeKey, VK_F8, VK_F9};
use rng::Rng;

const POLL_INTERVAL: Duration = Duration::from_millis(25);

fn main() {
    let cfg = match Config::parse(std::env::args().skip(1).collect()) {
        Ok(Some(cfg)) => cfg,
        Ok(None) => return,
        Err(err) => {
            eprintln!("error: {err}\n");
            config::print_help();
            std::process::exit(2);
        }
    };

    if cfg.probe {
        println!("{}", window::describe_foreground());
        return;
    }

    print_banner(&cfg);
    Session::new(cfg).run();
}

fn print_banner(cfg: &Config) {
    let key_names: Vec<&str> = cfg.keys.iter().map(|k| k.label.as_str()).collect();

    println!("anti-afk {}", env!("CARGO_PKG_VERSION"));
    println!(
        "  keys      : {} (held {}..{} ms)",
        key_names.join(", "),
        cfg.hold_min.as_millis(),
        cfg.hold_max.as_millis()
    );
    println!(
        "  interval  : {:.1}s .. {:.1}s, randomized every round",
        cfg.min_interval.as_secs_f64(),
        cfg.max_interval.as_secs_f64()
    );
    println!(
        "  focus     : {}",
        if cfg.require_focus {
            "only while Roblox is in the foreground"
        } else {
            "ignored (--any-window)"
        }
    );
    if cfg.jiggle {
        println!("  mouse     : small nudge and return");
    }
    if cfg.dry_run {
        println!("  mode      : dry run, no input will be sent");
    }
    println!();
}

struct Session {
    cfg: Config,
    rng: Rng,
    started: Instant,
    enabled: bool,
    next_action: Instant,
    waiting: bool,
    actions: u64,
}

impl Session {
    fn new(cfg: Config) -> Self {
        let mut rng = Rng::new();
        let delay = cfg.next_delay(&mut rng);

        Self {
            cfg,
            rng,
            started: Instant::now(),
            enabled: true,
            next_action: Instant::now() + delay,
            waiting: false,
            actions: 0,
        }
    }

    fn run(mut self) {
        let mut toggle = EdgeKey::new(VK_F8);
        let mut quit = EdgeKey::new(VK_F9);

        self.log("running - F8 pause/resume, F9 quit");

        loop {
            if quit.just_pressed() {
                self.log("F9 pressed - exiting");
                break;
            }

            if toggle.just_pressed() {
                self.enabled = !self.enabled;
                if self.enabled {
                    self.waiting = false;
                    self.next_action = Instant::now() + self.cfg.next_delay(&mut self.rng);
                    self.log(&format!("resumed - next input in {}", self.until_next()));
                } else {
                    self.log("paused");
                }
            }

            if self.enabled && Instant::now() >= self.next_action {
                self.tick();
            }

            sleep(POLL_INTERVAL);
        }

        self.log(&format!(
            "done - {} inputs sent over {}",
            self.actions,
            clock(self.started)
        ));
    }

    fn tick(&mut self) {
        if self.cfg.require_focus && !window::roblox_focused() {
            if !self.waiting {
                self.waiting = true;
                self.log("Roblox is not in the foreground - holding off");
            }
            return;
        }

        if self.waiting {
            self.waiting = false;
            self.log("Roblox is focused again");
        }

        let key = self.cfg.pick_key(&mut self.rng).clone();
        let hold = self.cfg.pick_hold(&mut self.rng);

        if self.cfg.dry_run {
            self.log(&format!("[dry-run] tap {} for {:?}", key.label, hold));
        } else {
            match input::tap_key(key.vk, hold, key.extended) {
                Ok(()) => {
                    self.actions += 1;
                    self.log(&format!("tap {} for {:?}", key.label, hold));
                }
                Err(err) => self.log(&format!("input failed: {err}")),
            }

            if self.cfg.jiggle {
                let dx = self.rng.range(2, 6) as i32 * sign(&mut self.rng);
                let dy = self.rng.range(2, 6) as i32 * sign(&mut self.rng);
                let pause = Duration::from_millis(self.rng.range(70, 150));
                input::nudge_mouse(dx, dy, pause);
            }
        }

        let delay = self.cfg.next_delay(&mut self.rng);
        self.next_action = Instant::now() + delay;
        self.log(&format!("next in {:.1}s", delay.as_secs_f64()));
    }

    fn until_next(&self) -> String {
        format!(
            "{:.1}s",
            self.next_action
                .saturating_duration_since(Instant::now())
                .as_secs_f64()
        )
    }

    fn log(&self, message: &str) {
        println!("[{}] {message}", clock(self.started));
    }
}

fn sign(rng: &mut Rng) -> i32 {
    if rng.chance(50) {
        1
    } else {
        -1
    }
}

fn clock(started: Instant) -> String {
    let total = started.elapsed().as_secs();
    format!(
        "{:02}:{:02}:{:02}",
        total / 3600,
        (total % 3600) / 60,
        total % 60
    )
}
