//! Command line options and the randomized timing policy.

use std::time::Duration;

use crate::keys::{self, KeySpec};
use crate::rng::Rng;

const DEFAULT_MIN_SECS: f64 = 45.0;
const DEFAULT_MAX_SECS: f64 = 120.0;
const DEFAULT_HOLD_MIN_MS: u64 = 40;
const DEFAULT_HOLD_MAX_MS: u64 = 90;

pub struct Config {
    pub min_interval: Duration,
    pub max_interval: Duration,
    pub keys: Vec<KeySpec>,
    pub hold_min: Duration,
    pub hold_max: Duration,
    pub jiggle: bool,
    pub require_focus: bool,
    pub dry_run: bool,
    pub probe: bool,
}

impl Config {
    /// `Ok(None)` means `--help` was handled.
    pub fn parse(args: Vec<String>) -> Result<Option<Self>, String> {
        let mut min_secs = DEFAULT_MIN_SECS;
        let mut max_secs = DEFAULT_MAX_SECS;
        let mut hold = (DEFAULT_HOLD_MIN_MS, DEFAULT_HOLD_MAX_MS);
        let mut keys: Vec<KeySpec> = Vec::new();
        let mut jiggle = false;
        let mut require_focus = true;
        let mut dry_run = false;
        let mut probe = false;

        let mut i = 0;
        while i < args.len() {
            let (flag, inline) = match args[i].split_once('=') {
                Some((flag, value)) => (flag.to_string(), Some(value.to_string())),
                None => (args[i].clone(), None),
            };

            match flag.as_str() {
                "-h" | "--help" => {
                    print_help();
                    return Ok(None);
                }
                "--min" => min_secs = parse_secs(&take(&flag, inline, &args, &mut i)?)?,
                "--max" => max_secs = parse_secs(&take(&flag, inline, &args, &mut i)?)?,
                "--hold" => hold = parse_hold(&take(&flag, inline, &args, &mut i)?)?,
                "--key" => {
                    let list = take(&flag, inline, &args, &mut i)?;
                    for part in list.split(',') {
                        keys.push(keys::parse(part)?);
                    }
                }
                "--jiggle" => jiggle = true,
                "--any-window" => require_focus = false,
                "--dry-run" => dry_run = true,
                "--probe" => probe = true,
                other => return Err(format!("unknown option '{other}'")),
            }

            i += 1;
        }

        if max_secs < min_secs {
            return Err(format!(
                "--max ({max_secs}s) must be greater than or equal to --min ({min_secs}s)"
            ));
        }
        if keys.is_empty() {
            keys.push(keys::parse("space")?);
        }

        Ok(Some(Self {
            min_interval: Duration::from_secs_f64(min_secs),
            max_interval: Duration::from_secs_f64(max_secs),
            keys,
            hold_min: Duration::from_millis(hold.0),
            hold_max: Duration::from_millis(hold.1),
            jiggle,
            require_focus,
            dry_run,
            probe,
        }))
    }

    /// Random gap between two inputs, with an occasional longer break.
    pub fn next_delay(&self, rng: &mut Rng) -> Duration {
        let lo = self.min_interval.as_millis() as u64;
        let hi = self.max_interval.as_millis() as u64;
        let mut ms = rng.range(lo, hi);

        if rng.chance(12) {
            ms += rng.range(lo / 2, lo.max(1));
        }

        Duration::from_millis(ms)
    }

    pub fn pick_key(&self, rng: &mut Rng) -> &KeySpec {
        &self.keys[rng.below(self.keys.len() as u64) as usize]
    }

    pub fn pick_hold(&self, rng: &mut Rng) -> Duration {
        let lo = self.hold_min.as_millis() as u64;
        let hi = self.hold_max.as_millis() as u64;
        Duration::from_millis(rng.range(lo, hi))
    }
}

fn take(
    flag: &str,
    inline: Option<String>,
    args: &[String],
    i: &mut usize,
) -> Result<String, String> {
    if let Some(value) = inline {
        if value.is_empty() {
            return Err(format!("{flag} needs a value"));
        }
        return Ok(value);
    }

    *i += 1;
    args.get(*i)
        .cloned()
        .ok_or_else(|| format!("{flag} needs a value"))
}

fn parse_secs(raw: &str) -> Result<f64, String> {
    let value: f64 = raw
        .trim()
        .parse()
        .map_err(|_| format!("'{raw}' is not a number"))?;

    if !value.is_finite() || value < 1.0 {
        return Err(format!("interval must be at least 1 second, got {raw}"));
    }
    if value > 3600.0 {
        return Err(format!("interval must be at most 3600 seconds, got {raw}"));
    }

    Ok(value)
}

fn parse_hold(raw: &str) -> Result<(u64, u64), String> {
    let (lo, hi) = match raw.split_once('-') {
        Some((lo, hi)) => (parse_ms(lo)?, parse_ms(hi)?),
        None => {
            let both = parse_ms(raw)?;
            (both, both)
        }
    };

    if hi < lo {
        return Err(format!("--hold range '{raw}' is backwards"));
    }

    Ok((lo, hi))
}

fn parse_ms(raw: &str) -> Result<u64, String> {
    let value: u64 = raw
        .trim()
        .parse()
        .map_err(|_| format!("'{raw}' is not a number of milliseconds"))?;

    if !(5..=3000).contains(&value) {
        return Err(format!("hold time must be 5-3000 ms, got {raw}"));
    }

    Ok(value)
}

pub fn print_help() {
    println!(
        "anti-afk {version} - keeps a Roblox session from being kicked for inactivity

USAGE:
    anti-afk [OPTIONS]

OPTIONS:
    --min <secs>        shortest gap between inputs          (default {min})
    --max <secs>        longest gap between inputs           (default {max})
    --key <list>        comma separated keys to tap          (default space)
                        names: space, enter, shift, ctrl, tab, esc,
                               w, a, s, d, e, q, f, 0-9, up, down, ...
    --hold <min-max>    how long each key is held, in ms     (default 40-90)
    --jiggle            also nudge the mouse and move it back
    --any-window        send input even when Roblox is not focused
    --dry-run           log what would happen, send nothing
    --probe             print the foreground window and exit
    -h, --help          show this help

HOTKEYS (work while Roblox has focus):
    F8                  pause / resume
    F9                  quit

NOTES:
    Input is only sent while the Roblox window is in the foreground unless
    --any-window is given. Intervals, keys and hold times are randomized so
    the pattern does not look like a fixed timer.
",
        version = env!("CARGO_PKG_VERSION"),
        min = DEFAULT_MIN_SECS as u64,
        max = DEFAULT_MAX_SECS as u64,
    );
}
