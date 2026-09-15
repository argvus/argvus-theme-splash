use std::{
    cell::RefCell,
    fs::File,
    io::Write,
    os::fd::FromRawFd,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use argvus_i18n::I18n;
use clap::Parser;
use gtk::{
    Align, Application, ApplicationWindow, Box as GtkBox, Label, Orientation, Spinner, gdk, glib,
    prelude::*,
};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

const APP_ID: &str = "org.argvus.ThemeSplash";
const DEFAULT_BACKGROUND: &str = "#101218";
const DEFAULT_FOREGROUND: &str = "#f4f4f4";
const DEFAULT_ACCENT: &str = "#7aa2f7";
const FADE_IN_MS: u32 = 180;
const FADE_OUT_MS: u32 = 220;

#[derive(Debug, Parser, Clone)]
#[command(
    name = "argvus-theme-splash",
    about = "Overlay shown while ARGVUS changes theme"
)]
struct Cli {
    #[arg(long, default_value = "ARGVUS")]
    theme: String,
    #[arg(long, default_value = DEFAULT_BACKGROUND)]
    background: String,
    #[arg(long, default_value = DEFAULT_FOREGROUND)]
    foreground: String,
    #[arg(long, default_value = DEFAULT_ACCENT)]
    accent: String,
    #[arg(long)]
    logo: Option<String>,
    /// Write READY after every monitor surface has been mapped.
    #[arg(long)]
    ready_file: Option<String>,
    /// Write READY to an inherited file descriptor after mapping.
    #[arg(long)]
    ready_fd: Option<i32>,
}

#[derive(Clone)]
struct Palette {
    background: String,
    foreground: String,
    accent: String,
}

impl Palette {
    fn from_cli(cli: &Cli) -> Self {
        let mut palette = Self {
            background: valid_color(&cli.background, DEFAULT_BACKGROUND),
            foreground: valid_color(&cli.foreground, DEFAULT_FOREGROUND),
            accent: valid_color(&cli.accent, DEFAULT_ACCENT),
        };
        if let Some((background, foreground)) = theme_colors(&cli.theme) {
            palette.background = background.to_owned();
            palette.foreground = foreground.to_owned();
        }
        palette
    }
}

fn theme_colors(theme: &str) -> Option<(&'static str, &'static str)> {
    let normalized = theme.trim().to_ascii_lowercase().replace(['_', ' '], "-");
    let normalized = normalized.strip_prefix("argvus-").unwrap_or(&normalized);
    let normalized = normalized.strip_suffix("-float").unwrap_or(normalized);

    match normalized {
        "dark-aether" => Some(("#191b27", "#3590bd")),
        "dark-silver" => Some(("#595959", "#333647")),
        "dark-slate" => Some(("#3b4352", "#7391a5")),
        "dark-universe" => Some(("#000000", "#ffffff")),
        "light-veil" => Some(("#ffffff", "#000000")),
        _ => None,
    }
}

fn valid_color(value: &str, fallback: &str) -> String {
    let valid = matches!(value.len(), 4 | 7 | 9)
        && value.starts_with('#')
        && value[1..]
            .chars()
            .all(|character| character.is_ascii_hexdigit());
    if valid {
        value.to_owned()
    } else {
        fallback.to_owned()
    }
}

fn css(palette: &Palette) -> String {
    format!(
        "window, #splash-surface, .splash-surface {{ background-color: {}; }} * {{ color: {}; }} .splash-accent {{ color: {}; }}",
        palette.background, palette.foreground, palette.accent
    )
}

fn tr_applying() -> String {
    I18n::new("appearance")
        .map(|catalog| catalog.tr("theme.applying"))
        .unwrap_or_else(|_| "theme.applying".to_owned())
}

fn main() -> glib::ExitCode {
    configure_parent_death();
    let cli = Cli::parse();
    let palette = Palette::from_cli(&cli);
    let application = Application::builder().application_id(APP_ID).build();
    application.connect_activate(move |application| {
        build_surfaces(application, &cli, &palette);
    });
    // GTK parses its own argv after clap. Pass only the application name so
    // splash-specific options are not rejected by GApplication.
    application.run_with_args(&[APP_ID])
}

fn configure_parent_death() {
    // A theme switch owns this process. If that owner disappears, ask the
    // normal SIGTERM path to fade the overlay instead of leaving it orphaned.
    unsafe {
        let _ = libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM);
        if libc::getppid() == 1 {
            let _ = libc::raise(libc::SIGTERM);
        }
    }
}

fn build_surfaces(application: &Application, cli: &Cli, palette: &Palette) {
    let display = gdk::Display::default().expect("a Wayland display is required");
    let monitors = display.monitors();
    let total = monitors.n_items();
    let surfaces: Rc<RefCell<Vec<ApplicationWindow>>> = Rc::new(RefCell::new(Vec::new()));

    for index in 0..total {
        let Some(item) = monitors.item(index) else {
            continue;
        };
        let Ok(monitor) = item.downcast::<gdk::Monitor>() else {
            continue;
        };
        let window = make_surface(application, &monitor, palette);
        window.present();
        surfaces.borrow_mut().push(window);
    }

    // The idle runs after present() has dispatched the map requests, which is
    // the point at which the compositor can display the layer-shell surfaces.
    let ready_file = cli.ready_file.clone();
    let ready_fd = cli.ready_fd;
    glib::idle_add_local_once(move || {
        signal_ready(ready_file.as_deref(), ready_fd);
    });

    let fade_state = Rc::new(RefCell::new(FadeState::new(surfaces.borrow().clone())));
    install_signal_handlers(&fade_state);
    start_fade_in(&fade_state);
}

fn make_surface(
    application: &Application,
    monitor: &gdk::Monitor,
    palette: &Palette,
) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .application(application)
        .decorated(false)
        .focusable(false)
        .focus_on_click(false)
        .build();
    window.init_layer_shell();
    window.set_namespace(Some("argvus-theme-splash"));
    window.set_layer(Layer::Overlay);
    window.set_monitor(Some(monitor));
    window.set_keyboard_mode(KeyboardMode::None);
    // Keep the surface in the compositor's full-output overlay area. Hyprland
    // otherwise places a zero-zone overlay below the exclusive Waybar strip.
    window.set_exclusive_zone(-1);
    for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
        window.set_anchor(edge, true);
        window.set_margin(edge, 0);
    }

    let provider = gtk::CssProvider::new();
    provider.load_from_string(&css(palette));
    gtk::style_context_add_provider_for_display(
        &gtk::prelude::WidgetExt::display(&window),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let content = GtkBox::new(Orientation::Vertical, 12);
    content.set_widget_name("splash-surface");
    content.add_css_class("splash-surface");
    content.set_halign(Align::Center);
    content.set_valign(Align::Center);
    content.set_hexpand(true);
    content.set_vexpand(true);
    content.set_can_focus(false);
    content.set_focusable(false);

    let spinner = Spinner::new();
    spinner.add_css_class("splash-accent");
    spinner.set_halign(Align::Center);
    spinner.set_width_request(24);
    spinner.set_height_request(24);
    spinner.start();
    let applying = Label::new(Some(&tr_applying()));
    applying.add_css_class("caption");
    applying.set_halign(Align::Center);
    content.append(&spinner);
    content.append(&applying);
    window.set_child(Some(&content));
    window.set_opacity(0.0);
    window
}

struct FadeState {
    windows: Vec<ApplicationWindow>,
    closing: bool,
}

impl FadeState {
    fn new(windows: Vec<ApplicationWindow>) -> Self {
        Self {
            windows,
            closing: false,
        }
    }
}

fn start_fade_in(state: &Rc<RefCell<FadeState>>) {
    animate(Rc::clone(state), 0.0, 1.0, FADE_IN_MS, false);
}

fn start_fade_out(state: &Rc<RefCell<FadeState>>) {
    let mut current = state.borrow_mut();
    if current.closing {
        return;
    }
    current.closing = true;
    drop(current);
    animate(Rc::clone(state), 1.0, 0.0, FADE_OUT_MS, true);
}

fn animate(state: Rc<RefCell<FadeState>>, from: f64, to: f64, duration_ms: u32, quit: bool) {
    let started = std::time::Instant::now();
    glib::timeout_add_local(Duration::from_millis(16), move || {
        let progress = (started.elapsed().as_millis() as f64 / duration_ms as f64).min(1.0);
        let value = from + (to - from) * progress;
        let finished = progress >= 1.0;
        for window in &state.borrow().windows {
            window.set_opacity(value);
        }
        if finished {
            if quit {
                for window in &state.borrow().windows {
                    window.close();
                }
                gtk::Application::default().quit();
            }
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });
}

fn install_signal_handlers(state: &Rc<RefCell<FadeState>>) {
    let requested = Arc::new(AtomicBool::new(false));
    for signal in [signal_hook::consts::SIGTERM, signal_hook::consts::SIGINT] {
        signal_hook::flag::register(signal, Arc::clone(&requested))
            .expect("signal handler registration must succeed");
    }
    let signal_state = Rc::clone(state);
    glib::timeout_add_local(Duration::from_millis(40), move || {
        if requested.load(Ordering::Relaxed) {
            start_fade_out(&signal_state);
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });
}

fn signal_ready(path: Option<&str>, fd: Option<i32>) {
    if let Some(path) = path
        && let Ok(mut file) = File::create(path)
    {
        let _ = file.write_all(b"READY\n");
    }
    if let Some(fd) = fd.filter(|value| *value >= 0) {
        // The descriptor is explicitly handed to this process by the caller.
        let mut file = unsafe { File::from_raw_fd(fd) };
        let _ = file.write_all(b"READY\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_supported_hex_colors() {
        assert_eq!(valid_color("#abc", DEFAULT_BACKGROUND), "#abc");
        assert_eq!(valid_color("#AABBCC", DEFAULT_BACKGROUND), "#AABBCC");
        assert_eq!(valid_color("#AABBCCDD", DEFAULT_BACKGROUND), "#AABBCCDD");
    }

    #[test]
    fn rejects_css_injection_and_bad_lengths() {
        assert_eq!(valid_color("red", DEFAULT_BACKGROUND), DEFAULT_BACKGROUND);
        assert_eq!(
            valid_color("#123456;", DEFAULT_BACKGROUND),
            DEFAULT_BACKGROUND
        );
        assert_eq!(
            valid_color("#12345", DEFAULT_BACKGROUND),
            DEFAULT_BACKGROUND
        );
    }

    #[test]
    fn cli_defaults_are_safe() {
        let cli = Cli::try_parse_from(["argvus-theme-splash"]).unwrap();
        assert_eq!(Palette::from_cli(&cli).background, DEFAULT_BACKGROUND);
        assert_eq!(Palette::from_cli(&cli).foreground, DEFAULT_FOREGROUND);
        assert_eq!(Palette::from_cli(&cli).accent, DEFAULT_ACCENT);
    }

    #[test]
    fn known_theme_names_select_their_main_colors() {
        let cases = [
            ("ARGVUS Dark Aether", "#191b27", "#3590bd"),
            ("argvus-dark-silver-float", "#595959", "#333647"),
            ("ARGVUS Dark Slate", "#3b4352", "#7391a5"),
            ("argvus-dark-universe", "#000000", "#ffffff"),
            ("ARGVUS Light Veil", "#ffffff", "#000000"),
        ];

        for (theme, background, foreground) in cases {
            let cli = Cli::try_parse_from([
                "argvus-theme-splash",
                "--theme",
                theme,
                "--background",
                "#abcdef",
                "--foreground",
                "#123456",
            ])
            .unwrap();
            let palette = Palette::from_cli(&cli);
            assert_eq!(palette.background, background);
            assert_eq!(palette.foreground, foreground);
        }
    }

    #[test]
    fn unknown_theme_keeps_explicit_cli_colors() {
        let cli = Cli::try_parse_from([
            "argvus-theme-splash",
            "--theme",
            "custom-theme",
            "--background",
            "#abcdef",
            "--foreground",
            "#123456",
        ])
        .unwrap();
        let palette = Palette::from_cli(&cli);
        assert_eq!(palette.background, "#abcdef");
        assert_eq!(palette.foreground, "#123456");
    }
}
