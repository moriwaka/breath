use std::{cell::RefCell, path::PathBuf, rc::Rc, time::Duration};

use adw::prelude::*;
use breath::{AudioMode, PresetId, Session, SessionLength, SessionStatus, StepKind, preset_by_id};
use gst::prelude::*;
const APP_ID: &str = "io.github.moriwaka.Breath";
const AUDIO_DIR: &str = "/usr/share/breath/audio";

fn main() {
    gst::init().expect("GStreamer must initialize");
    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_home);
    app.run();
}

fn build_home(app: &adw::Application) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Breath")
        .default_width(520)
        .default_height(640)
        .build();
    let settings = gtk::gio::Settings::new(APP_ID);
    show_home(&window, &settings);
    window.present();
}

fn show_home(window: &adw::ApplicationWindow, settings: &gtk::gio::Settings) {
    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&adw::WindowTitle::new(
        "Breath",
        "呼吸に意識を向けましょう",
    )));

    let list = gtk::ListBox::new();
    list.add_css_class("boxed-list");
    for id in PresetId::ALL {
        let preset = preset_by_id(id);
        let row = adw::ActionRow::builder()
            .title(japanese_name(id))
            .subtitle(format!(
                "{}  ·  {}",
                japanese_description(id),
                format_steps(preset.steps)
            ))
            .build();
        let start = gtk::Button::with_label("開始");
        start.add_css_class("suggested-action");
        let window = window.clone();
        let settings = settings.clone();
        start.connect_clicked(move |_| show_session(&window, &settings, id));
        row.add_suffix(&start);
        row.set_activatable_widget(Some(&start));
        list.append(&row);
    }

    let content = gtk::Box::new(gtk::Orientation::Vertical, 18);
    content.set_margin_top(24);
    content.set_margin_bottom(24);
    content.set_margin_start(24);
    content.set_margin_end(24);
    content.append(&list);

    let duration = gtk::Label::new(Some(&format!(
        "セッション時間: {}（設定は次回起動時に保存されます）",
        format_session_length(session_length(settings))
    )));
    duration.add_css_class("dim-label");
    duration.set_halign(gtk::Align::Start);
    content.append(&duration);

    let preferences = gtk::Button::with_label("設定");
    preferences.set_halign(gtk::Align::Start);
    let window_for_preferences = window.clone();
    let settings_for_preferences = settings.clone();
    preferences.connect_clicked(move |_| {
        show_preferences(&window_for_preferences, &settings_for_preferences)
    });
    content.append(&preferences);

    let layout = gtk::Box::new(gtk::Orientation::Vertical, 0);
    layout.append(&header);
    layout.append(&content);
    window.set_content(Some(&layout));
}

fn show_session(window: &adw::ApplicationWindow, settings: &gtk::gio::Settings, id: PresetId) {
    let preset = preset_by_id(id);
    let session = Rc::new(RefCell::new(Session::start(
        preset,
        session_length(settings).as_option_ms(),
    )));
    let audio = Rc::new(AudioPlayer::default());
    let phase = gtk::Label::new(Some("吸う"));
    phase.add_css_class("title-1");
    let remaining = gtk::Label::new(None);
    remaining.add_css_class("title-3");
    let guide = gtk::DrawingArea::new();
    guide.set_content_width(260);
    guide.set_content_height(260);
    guide.set_draw_func(|_, cr, width, height| {
        let radius = f64::from(width.min(height)) * 0.31;
        cr.set_source_rgba(0.18, 0.47, 0.56, 0.85);
        cr.arc(
            f64::from(width) / 2.0,
            f64::from(height) / 2.0,
            radius,
            0.0,
            std::f64::consts::TAU,
        );
        let _ = cr.fill();
    });

    let pause = gtk::Button::with_label("一時停止");
    let stop = gtk::Button::with_label("停止");
    stop.add_css_class("destructive-action");
    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    controls.set_halign(gtk::Align::Center);
    controls.append(&pause);
    controls.append(&stop);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 18);
    content.set_valign(gtk::Align::Center);
    content.set_halign(gtk::Align::Center);
    content.append(&gtk::Label::new(Some(japanese_name(id))));
    content.append(&guide);
    content.append(&phase);
    content.append(&remaining);
    content.append(&controls);
    window.set_content(Some(&content));

    if let Some((kind, _)) = session.borrow().current_step() {
        play_step_cue(&audio, settings, kind);
    }

    let timer_session = session.clone();
    let timer_phase = phase.clone();
    let timer_remaining = remaining.clone();
    let timer_guide = guide.clone();
    let timer_audio = audio.clone();
    let timer_settings = settings.clone();
    let mut announced_step = session.borrow().current_step().map(|(kind, _)| kind);
    gtk::glib::timeout_add_local(Duration::from_millis(100), move || {
        let mut current = timer_session.borrow_mut();
        current.advance(100);
        if current.status() == SessionStatus::Stopped {
            return gtk::glib::ControlFlow::Break;
        }
        if let Some((kind, _)) = current.current_step() {
            timer_phase.set_label(japanese_step(kind));
            if announced_step != Some(kind) {
                play_step_cue(&timer_audio, &timer_settings, kind);
                announced_step = Some(kind);
            }
        }
        timer_remaining.set_label(&format_remaining(current.session_remaining_ms()));
        timer_guide.queue_draw();
        if current.status() == SessionStatus::Completed {
            timer_phase.set_label("完了しました");
            timer_audio.play("endingbell1.mp3");
            return gtk::glib::ControlFlow::Break;
        }
        gtk::glib::ControlFlow::Continue
    });

    let pause_session = session.clone();
    pause.connect_clicked(move |button| {
        let mut current = pause_session.borrow_mut();
        if current.status() == SessionStatus::Running {
            current.pause();
            button.set_label("再開");
        } else if current.status() == SessionStatus::Paused {
            current.resume();
            button.set_label("一時停止");
        }
    });
    let stop_session = session.clone();
    let stop_audio = audio.clone();
    let window_for_stop = window.clone();
    let settings_for_stop = settings.clone();
    stop.connect_clicked(move |_| {
        stop_session.borrow_mut().stop();
        stop_audio.stop();
        show_home(&window_for_stop, &settings_for_stop);
    });
}

fn session_length(settings: &gtk::gio::Settings) -> SessionLength {
    SessionLength::from_minutes(
        settings
            .uint("session-minutes")
            .min(u32::from(SessionLength::MAX_MINUTES)) as u8,
    )
}

fn format_session_length(length: SessionLength) -> String {
    match length.minutes() {
        0 => "無制限".to_string(),
        minutes => format!("{minutes} 分"),
    }
}

fn show_preferences(window: &adw::ApplicationWindow, settings: &gtk::gio::Settings) {
    let dialog = adw::PreferencesDialog::new();
    let page = adw::PreferencesPage::new();
    let group = adw::PreferencesGroup::new();
    group.set_title("セッション");

    let duration = adw::SpinRow::with_range(0.0, f64::from(SessionLength::MAX_MINUTES), 1.0);
    duration.set_title("セッション時間（分）");
    duration.set_value(f64::from(session_length(settings).minutes()));
    duration.set_subtitle("0 分は無制限です");
    let settings_for_duration = settings.clone();
    duration.connect_value_notify(move |row| {
        let minutes = row
            .value()
            .round()
            .clamp(0.0, f64::from(SessionLength::MAX_MINUTES));
        let _ = settings_for_duration.set_uint("session-minutes", minutes as u32);
    });
    group.add(&duration);

    let voice = adw::ComboRow::new();
    voice.set_title("ガイド音声");
    voice.set_model(Some(&gtk::StringList::new(&[
        "Paul", "Laura", "ベル", "オフ",
    ])));
    voice.set_selected(audio_mode_index(AudioMode::from_key(
        settings.string("audio-mode").as_str(),
    )));
    let settings_for_voice = settings.clone();
    voice.connect_selected_notify(move |row| {
        let _ =
            settings_for_voice.set_string("audio-mode", audio_mode_for_index(row.selected()).key());
    });
    group.add(&voice);
    page.add(&group);
    dialog.add(&page);
    dialog.present(Some(window));
}

fn audio_mode_index(mode: AudioMode) -> u32 {
    match mode {
        AudioMode::Paul => 0,
        AudioMode::Laura => 1,
        AudioMode::Bell => 2,
        AudioMode::Off => 3,
    }
}

fn audio_mode_for_index(index: u32) -> AudioMode {
    match index {
        1 => AudioMode::Laura,
        2 => AudioMode::Bell,
        3 => AudioMode::Off,
        _ => AudioMode::Paul,
    }
}

#[derive(Default)]
struct AudioPlayer {
    current: RefCell<Option<gst::Element>>,
}

impl AudioPlayer {
    fn play(&self, asset: &str) {
        self.stop();
        let path = audio_path(asset);
        if !path.is_file() {
            return;
        }
        let uri = gtk::gio::File::for_path(path).uri();
        let Ok(player) = gst::ElementFactory::make("playbin")
            .property_from_str("uri", uri.as_str())
            .build()
        else {
            return;
        };
        if player.set_state(gst::State::Playing).is_ok() {
            self.current.replace(Some(player));
        }
    }

    fn stop(&self) {
        if let Some(player) = self.current.borrow_mut().take() {
            let _ = player.set_state(gst::State::Null);
        }
    }
}

fn play_step_cue(audio: &AudioPlayer, settings: &gtk::gio::Settings, step: StepKind) {
    if let Some(asset) =
        AudioMode::from_key(settings.string("audio-mode").as_str()).asset_for_step(step)
    {
        audio.play(asset);
    }
}

fn audio_path(asset: &str) -> PathBuf {
    let installed = PathBuf::from(AUDIO_DIR).join(asset);
    installed
        .is_file()
        .then_some(installed)
        .unwrap_or_else(|| PathBuf::from("assets/audio").join(asset))
}

fn japanese_name(id: PresetId) -> &'static str {
    match id {
        PresetId::DeepCalm => "4-7-8 深い落ち着き",
        PresetId::Awake => "目覚め",
        PresetId::Coherent => "コヒーレント呼吸",
        PresetId::ExtendedExhale => "長い呼気",
        PresetId::Pranayama => "プラーナヤーマ",
        PresetId::Square => "スクエア呼吸",
        PresetId::Ujjayi => "ウジャイ呼吸",
    }
}

fn japanese_description(id: PresetId) -> &'static str {
    match id {
        PresetId::DeepCalm => "ゆっくりと神経を落ち着かせます",
        PresetId::Awake => "朝の目覚めと集中に",
        PresetId::Coherent => "等しいリズムで整えます",
        PresetId::ExtendedExhale => "短時間で落ち着きたいときに",
        PresetId::Pranayama => "ヨガの基本的な呼吸法です",
        PresetId::Square => "一定のリズムで呼吸します",
        PresetId::Ujjayi => "心身のバランスを整えます",
    }
}

fn japanese_step(step: StepKind) -> &'static str {
    match step {
        StepKind::Inhale => "吸う",
        StepKind::HoldAfterInhale => "止める",
        StepKind::Exhale => "吐く",
        StepKind::HoldAfterExhale => "止める",
    }
}

fn format_steps(steps: [u32; 4]) -> String {
    steps
        .into_iter()
        .filter(|value| *value > 0)
        .map(|value| format!("{}秒", value / 1_000))
        .collect::<Vec<_>>()
        .join(" / ")
}

fn format_remaining(remaining: Option<u32>) -> String {
    match remaining {
        Some(milliseconds) => format!(
            "残り {}:{:02}",
            milliseconds / 60_000,
            (milliseconds / 1_000) % 60
        ),
        None => "無制限セッション".to_string(),
    }
}
