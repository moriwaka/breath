use std::{cell::RefCell, rc::Rc, time::Duration};

use adw::prelude::*;
use breath::{PresetId, Session, SessionLength, SessionStatus, StepKind, preset_by_id};
const APP_ID: &str = "io.github.moriwaka.Breath";

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
    show_home(&window);
    window.present();
}

fn show_home(window: &adw::ApplicationWindow) {
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
        start.connect_clicked(move |_| show_session(&window, id));
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

    let duration = gtk::Label::new(Some(
        "セッション時間: 5 分（設定は次回起動時に保存されます）",
    ));
    duration.add_css_class("dim-label");
    duration.set_halign(gtk::Align::Start);
    content.append(&duration);

    let layout = gtk::Box::new(gtk::Orientation::Vertical, 0);
    layout.append(&header);
    layout.append(&content);
    window.set_content(Some(&layout));
}

fn show_session(window: &adw::ApplicationWindow, id: PresetId) {
    let preset = preset_by_id(id);
    let session = Rc::new(RefCell::new(Session::start(
        preset,
        SessionLength::default().as_option_ms(),
    )));
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

    let timer_session = session.clone();
    let timer_phase = phase.clone();
    let timer_remaining = remaining.clone();
    let timer_guide = guide.clone();
    gtk::glib::timeout_add_local(Duration::from_millis(100), move || {
        let mut current = timer_session.borrow_mut();
        current.advance(100);
        if let Some((kind, _)) = current.current_step() {
            timer_phase.set_label(japanese_step(kind));
        }
        timer_remaining.set_label(&format_remaining(current.session_remaining_ms()));
        timer_guide.queue_draw();
        if current.status() == SessionStatus::Completed {
            timer_phase.set_label("完了しました");
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
    let window_for_stop = window.clone();
    stop.connect_clicked(move |_| show_home(&window_for_stop));
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
