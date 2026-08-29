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

    let clamp = adw::Clamp::builder()
        .maximum_size(680)
        .child(&content)
        .build();
    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&clamp)
        .build();
    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&scrolled));
    window.set_content(Some(&toolbar));
}

fn show_session(window: &adw::ApplicationWindow, settings: &gtk::gio::Settings, id: PresetId) {
    let preset = preset_by_id(id);
    let session = Rc::new(RefCell::new(Session::start_countdown(
        preset,
        session_length(settings).as_option_ms(),
        3_000,
    )));
    let audio = Rc::new(AudioPlayer::default());
    let phase = gtk::Label::new(Some("吸う"));
    phase.add_css_class("title-1");
    let remaining = gtk::Label::new(None);
    remaining.add_css_class("title-2");
    remaining.set_visible(false);
    let countdown = gtk::Label::new(Some("3"));
    countdown.add_css_class("title-1");
    let hint = gtk::Label::new(Some("準備しましょう"));
    hint.add_css_class("dim-label");
    hint.set_wrap(true);
    let audio_warning = gtk::Label::new(None);
    audio_warning.add_css_class("warning");
    audio_warning.set_wrap(true);
    audio_warning.set_visible(false);
    let guide_progress = Rc::new(RefCell::new(0.0));
    let guide_kind = Rc::new(RefCell::new(StepKind::Inhale));
    let guide = gtk::DrawingArea::new();
    guide.set_content_width(260);
    guide.set_content_height(260);
    let draw_progress = guide_progress.clone();
    let draw_kind = guide_kind.clone();
    guide.set_draw_func(move |_, cr, width, height| {
        let progress = *draw_progress.borrow();
        let kind = *draw_kind.borrow();
        let scale = match kind {
            StepKind::Inhale => 0.58 + progress * 0.42,
            StepKind::Exhale => 1.0 - progress * 0.42,
            StepKind::HoldAfterInhale | StepKind::HoldAfterExhale => 1.0,
        };
        let max_radius = f64::from(width.min(height)) * 0.31;
        cr.set_source_rgba(0.18, 0.47, 0.56, 0.18);
        cr.set_line_width(2.0);
        cr.arc(
            f64::from(width) / 2.0,
            f64::from(height) / 2.0,
            max_radius,
            0.0,
            std::f64::consts::TAU,
        );
        let _ = cr.stroke();
        let radius = max_radius * scale;
        cr.set_source_rgba(0.18, 0.47, 0.56, 0.85);
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
    pause.set_sensitive(false);
    let stop = gtk::Button::with_label("停止");
    stop.add_css_class("destructive-action");
    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    controls.set_halign(gtk::Align::Center);
    controls.append(&pause);
    controls.append(&stop);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 18);
    content.set_margin_top(24);
    content.set_margin_bottom(24);
    content.set_margin_start(24);
    content.set_margin_end(24);
    content.set_valign(gtk::Align::Center);
    content.set_halign(gtk::Align::Center);
    let name = gtk::Label::new(Some(japanese_name(id)));
    name.add_css_class("title-2");
    content.append(&name);
    content.append(&guide);
    content.append(&countdown);
    content.append(&phase);
    content.append(&hint);
    content.append(&audio_warning);
    content.append(&remaining);
    content.append(&controls);
    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&adw::WindowTitle::new("Breath", japanese_name(id))));
    let back = gtk::Button::from_icon_name("go-previous-symbolic");
    back.set_tooltip_text(Some("ホームに戻る"));
    header.pack_start(&back);
    let clamp = adw::Clamp::builder()
        .maximum_size(680)
        .child(&content)
        .build();
    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(&clamp));
    window.set_content(Some(&toolbar));

    let timer_session = session.clone();
    let timer_phase = phase.clone();
    let timer_hint = hint.clone();
    let timer_pause = pause.clone();
    let timer_remaining = remaining.clone();
    let timer_countdown = countdown.clone();
    let timer_guide = guide.clone();
    let timer_audio = audio.clone();
    let timer_settings = settings.clone();
    let timer_audio_warning = audio_warning.clone();
    let timer_progress = guide_progress.clone();
    let timer_kind = guide_kind.clone();
    let mut announced_step = session.borrow().current_step().map(|(kind, _)| kind);
    gtk::glib::timeout_add_local(Duration::from_millis(100), move || {
        let mut current = timer_session.borrow_mut();
        current.advance(100);
        if current.status() == SessionStatus::Stopped {
            return gtk::glib::ControlFlow::Break;
        }
        if current.status() == SessionStatus::Countdown {
            timer_countdown.set_visible(true);
            timer_remaining.set_visible(false);
            timer_phase.set_label("準備");
            timer_hint.set_label("まもなく開始します");
            timer_countdown.set_label(&format_countdown(current.countdown_remaining_ms()));
        } else if let Some((kind, _)) = current.current_step() {
            timer_countdown.set_visible(false);
            timer_remaining.set_visible(true);
            timer_pause.set_sensitive(true);
            timer_phase.set_label(japanese_step(kind));
            timer_hint.set_label(japanese_hint(kind));
            *timer_progress.borrow_mut() = current.phase_progress().unwrap_or(0.0);
            *timer_kind.borrow_mut() = kind;
            if announced_step != Some(kind) {
                if !play_step_cue(&timer_audio, &timer_settings, kind) {
                    timer_audio_warning.set_label(
                        "音声を再生できません。音声ファイルまたはGStreamerのデコーダーを確認してください。",
                    );
                    timer_audio_warning.set_visible(true);
                }
                announced_step = Some(kind);
            }
        }
        timer_remaining.set_label(&format_remaining(current.session_remaining_ms()));
        timer_guide.queue_draw();
        if current.status() == SessionStatus::Completed {
            timer_phase.set_label("完了しました");
            let mode = AudioMode::from_key(timer_settings.string("audio-mode").as_str());
            if mode.plays_completion_cue() && !timer_audio.play("endingbell1.mp3") {
                timer_audio_warning.set_label(
                    "完了音を再生できません。音声ファイルまたはGStreamerのデコーダーを確認してください。",
                );
                timer_audio_warning.set_visible(true);
            }
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
    let back_session = session.clone();
    let back_audio = audio.clone();
    let window_for_back = window.clone();
    let settings_for_back = settings.clone();
    back.connect_clicked(move |_| {
        back_session.borrow_mut().stop();
        back_audio.stop();
        show_home(&window_for_back, &settings_for_back);
    });

    let keyboard_session = session.clone();
    let keyboard_audio = audio.clone();
    let window_for_keyboard = window.downgrade();
    let settings_for_keyboard = settings.clone();
    let keyboard = gtk::EventControllerKey::new();
    keyboard.connect_key_pressed(move |_, key, _, _| {
        if key == gtk::gdk::Key::Escape {
            keyboard_session.borrow_mut().stop();
            keyboard_audio.stop();
            if let Some(window) = window_for_keyboard.upgrade() {
                show_home(&window, &settings_for_keyboard);
            }
            gtk::glib::Propagation::Stop
        } else {
            gtk::glib::Propagation::Proceed
        }
    });
    toolbar.add_controller(keyboard);
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
    fn play(&self, asset: &str) -> bool {
        self.stop();
        let path = audio_path(asset);
        if !path.is_file() {
            return false;
        }
        let uri = gtk::gio::File::for_path(path).uri();
        let Ok(player) = gst::ElementFactory::make("playbin")
            .property_from_str("uri", uri.as_str())
            .build()
        else {
            return false;
        };
        if player.set_state(gst::State::Playing).is_ok() {
            self.current.replace(Some(player));
            true
        } else {
            false
        }
    }

    fn stop(&self) {
        if let Some(player) = self.current.borrow_mut().take() {
            let _ = player.set_state(gst::State::Null);
        }
    }
}

fn play_step_cue(audio: &AudioPlayer, settings: &gtk::gio::Settings, step: StepKind) -> bool {
    if let Some(asset) =
        AudioMode::from_key(settings.string("audio-mode").as_str()).asset_for_step(step)
    {
        audio.play(asset)
    } else {
        true
    }
}

fn audio_path(asset: &str) -> PathBuf {
    let installed = PathBuf::from(AUDIO_DIR).join(asset);
    if installed.is_file() {
        installed
    } else {
        PathBuf::from("assets/audio").join(asset)
    }
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

fn format_countdown(remaining: Option<u32>) -> String {
    let seconds = remaining.unwrap_or(0).div_ceil(1_000);
    format!("開始まで {seconds}")
}

fn japanese_hint(step: StepKind) -> &'static str {
    match step {
        StepKind::Inhale => "ゆっくり大きく",
        StepKind::Exhale => "ゆっくり小さく",
        StepKind::HoldAfterInhale | StepKind::HoldAfterExhale => "そのまま保ちます",
    }
}
