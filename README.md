# anti-afk

<<<<<<< HEAD
[![latest release](https://img.shields.io/github/v/release/VoidSchema/ant-afk?label=release)](https://github.com/VoidSchema/ant-afk/releases/latest)
=======
[![latest release](https://img.shields.io/github/v/release/VoidSchema/ant-afk?label=release)](https://github.com/VoidSchema/anti-afk/releases/latest)
>>>>>>> 112b6593ef82e16a7c020f6e02f7ec49c21b901b

A small Rust tool for Windows that keeps your Roblox session from being kicked for going idle. It taps a key every so often, with enough randomness that the pattern stops looking like a metronome.

Roblox drops idle clients after about 20 minutes. This keeps a keypress in the air well before the timer runs out.

## Download

Grab `anti-afk.exe` from the [latest release](https://github.com/VoidSchema/ant-afk/releases/latest/download/anti-afk.exe). It is one file of about 180 KB for 64-bit Windows 10 and 11, with no installer and no dependencies.

Double-click it, leave the console window open, and go do the thing you were going to do. The console logs every tap with a timestamp, so you can see it is still awake. `F8` pauses, `F9` closes.

The exe is not code-signed, so SmartScreen may introduce itself with a blue "Windows protected your PC" box. Click "More info", then "Run anyway". That is the entire installation.

## How it behaves

The tool checks which window is in front. If Roblox is there, it taps one key from your list, holds it for 40 to 90 ms, then goes quiet for 45 to 120 seconds. The key and the timing are chosen fresh every round.

Roughly one time in eight, the pause stretches longer than usual. That part is deliberate. A keep-alive that fires on the same beat for three hours is easy to spot, and honestly a little sad.

Key presses go out through `SendInput` as hardware scan codes rather than virtual keys, because that is what games actually read. There is an optional mouse nudge, off by default: a camera that twitches on its own is worse than a session that quietly times out.

By default, input is only sent while the Roblox window has focus, so nothing leaks into Discord or your browser after you alt-tab.

## What it will not do

It will not play the game for you. There is no movement loop, no clicking, and no attempt to hide anything from Roblox. It taps one key every minute or two: enough to reset the idle timer, and not much else.

## Usage

```powershell
# space bar, every 45 to 120 seconds, only while Roblox is in front
.\anti-afk.exe

# see what it would do without sending anything
.\anti-afk.exe --dry-run

# a few keys, shorter pauses, plus the mouse nudge
.\anti-afk.exe --key space,w,a,s,d --min 30 --max 90 --jiggle

# check that Roblox is detected properly
.\anti-afk.exe --probe
```

### Options

| Option | What it does | Default |
| --- | --- | --- |
| `--min <seconds>` | shortest pause between inputs | 45 |
| `--max <seconds>` | longest pause between inputs | 120 |
| `--key <list>` | keys to tap, comma separated | `space` |
| `--hold <min-max>` | how long each key is held, in ms | `40-90` |
| `--jiggle` | move the mouse 2 to 6 px and put it back | off |
| `--any-window` | send input even when Roblox is not focused | off |
| `--dry-run` | log everything, send nothing | off |
| `--probe` | print the foreground process and title, then exit | - |

Key names: `space`, `enter`, `tab`, `shift`, `ctrl`, `alt`, `esc`, `backspace`, `up`, `down`, `left`, `right`, `insert`, `delete`, `home`, `end`, `pageup`, `pagedown`, any letter `a` to `z`, and any digit `0` to `9`.

### Hotkeys

| Key | Action |
| --- | --- |
| `F8` | pause or resume |
| `F9` | quit |

Both are global hotkeys, so they work with Roblox in front and you never have to alt-tab to stop the tool.

## Build from source

If you would rather compile it yourself:

```powershell
cargo build --release
```

You need a Rust toolchain on Windows. When the build finishes with `Finished`, you have `target\release\anti-afk.exe`, the same binary that ships in the release. No runtime, no DLLs, nothing else to install.

## Before you leave it running

Run the tool at the same privilege level as Roblox. If the game is running as administrator, a normal process cannot send it input, because Windows blocks input across integrity levels (UIPI).

Space makes your character jump. If that gets in the way, pick a key the game ignores, or move the jump key somewhere harmless.

If a chat box is open when a tap lands, the game receives a space in your message. Press `F8` before you start typing. The tool waits patiently while it is paused.

If `--probe` reports `roblox: no` while the game is clearly in front, the window title is probably not what the tool expects (some Microsoft Store builds behave this way). Use `--any-window`, or send the probe output along with a bug report.

There is nothing to uninstall. Close the console window, delete the exe, and the tool leaves no config files or registry keys behind.

Randomized timing makes the tool less robotic. It does not make the input invisible: Windows still marks injected events as injected, and Roblox can still treat automation as a violation of its terms. This is a keep-alive for your own session, not a bypass for anything.

Made for the very specific problem of wanting to step away for a few minutes without starting over. Tea break approved.
