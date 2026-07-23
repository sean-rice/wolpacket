mod app;
mod ui;

use app::App;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use std::io::{self, stdout};
use std::process;
use wolpacket_core::wake;

const HELP: &str = "\
Wake-On-LAN command and TUI: wake devices on your LAN.

Usage: wolpacket [OPTIONS]

Options:
  -d, --device <ID>   Wake a specific device by id (e.g. \"sonos-move2\") and exit
  -h, --help          Print this help message
  -V, --version       Print version
";

fn main() -> io::Result<()> {
    let mut args = pico_args::Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        print!("{HELP}");
        process::exit(0);
    }

    if args.contains(["-V", "--version"]) {
        println!("wolpacket {}", env!("CARGO_PKG_VERSION"));
        process::exit(0);
    }

    let device_id: Option<String> = args.opt_value_from_str("--device").unwrap_or_else(|e| {
        eprintln!("error: {}", e);
        process::exit(1);
    });
    // Also accept -d as short form.
    let device_id = device_id.or_else(|| args.opt_value_from_str("-d").ok().flatten());

    let remaining = args.finish();
    if !remaining.is_empty() {
        eprintln!(
            "error: unexpected argument '{}'",
            remaining[0].to_string_lossy()
        );
        eprintln!("Run 'wolpacket --help' for usage.");
        process::exit(1);
    }

    // Headless mode: skip the TUI, send the packet, print result, exit.
    if let Some(id) = device_id {
        let app = App::new();
        let device = app.devices.iter().find(|d| d.id == id).unwrap_or_else(|| {
            eprintln!("error: no device with id \"{}\"", id);
            process::exit(1);
        });

        let broadcast = app.broadcast_addr();
        let addr = format!("{}:9", broadcast);
        match wake(device.mac, &addr) {
            Ok(()) => {
                println!(
                    "Woke {} ({}): packet sent to {}",
                    device.name, device.mac, addr
                );
                process::exit(0);
            }
            Err(e) => {
                eprintln!("Error waking {}: {}", device.name, e);
                process::exit(1);
            }
        }
    }

    // TUI mode.
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    run_event_loop(&mut terminal, &mut app)?;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}

fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| ui::render(frame, app))?;

        if !event::poll(std::time::Duration::from_millis(100))? {
            continue;
        }

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }

            if app.editing_lan {
                match key.code {
                    KeyCode::Enter => {
                        app.confirm_edit_lan();
                    }
                    KeyCode::Esc => {
                        app.cancel_edit_lan();
                    }
                    KeyCode::Char(c) => {
                        app.push_lan_char(c);
                    }
                    KeyCode::Backspace => {
                        app.pop_lan_char();
                    }
                    _ => {}
                }
                continue;
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    app.should_quit = true;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    app.select_next();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    app.select_prev();
                }
                KeyCode::Char('e') => {
                    app.start_edit_lan();
                }
                KeyCode::Enter => {
                    if let Some(device) = app.selected_device() {
                        let name = device.name.clone();
                        let mac = device.mac;
                        let broadcast = app.broadcast_addr();
                        let addr = format!("{}:9", broadcast);
                        match wake(mac, &addr) {
                            Ok(()) => {
                                app.set_status(
                                    format!("Woke {}: packet sent to {}", name, addr),
                                    true,
                                );
                            }
                            Err(e) => {
                                app.set_status(format!("Error waking {}: {}", name, e), false);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}
