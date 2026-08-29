use breath::{PresetId, Session, SessionStatus, StepKind, preset_by_id};

#[test]
fn running_session_moves_to_the_next_non_zero_phase() {
    let mut session = Session::start(preset_by_id(PresetId::Awake), None);

    assert_eq!(session.status(), SessionStatus::Running);
    assert_eq!(session.current_step(), Some((StepKind::Inhale, 6_000)));
    assert_eq!(session.phase_remaining_ms(), Some(6_000));

    session.advance(6_000);

    assert_eq!(session.current_step(), Some((StepKind::Exhale, 2_000)));
    assert_eq!(session.phase_remaining_ms(), Some(2_000));
}

#[test]
fn paused_session_does_not_consume_time_until_resumed() {
    let mut session = Session::start(preset_by_id(PresetId::Square), Some(300_000));

    session.advance(1_500);
    session.pause();
    session.advance(10_000);

    assert_eq!(session.status(), SessionStatus::Paused);
    assert_eq!(session.phase_remaining_ms(), Some(2_500));
    assert_eq!(session.session_remaining_ms(), Some(298_500));

    session.resume();
    session.advance(2_500);

    assert_eq!(
        session.current_step(),
        Some((StepKind::HoldAfterInhale, 4_000))
    );
}

#[test]
fn timed_session_completes_at_the_configured_limit() {
    let mut session = Session::start(preset_by_id(PresetId::Square), Some(5_000));

    session.advance(4_999);
    assert_eq!(session.status(), SessionStatus::Running);
    assert_eq!(session.session_remaining_ms(), Some(1));

    session.advance(1);

    assert_eq!(session.status(), SessionStatus::Completed);
    assert_eq!(session.current_step(), None);
    assert_eq!(session.session_remaining_ms(), Some(0));
}

#[test]
fn unlimited_session_keeps_running_across_full_cycles() {
    let mut session = Session::start(preset_by_id(PresetId::Square), None);

    session.advance(16_000);

    assert_eq!(session.status(), SessionStatus::Running);
    assert_eq!(session.current_step(), Some((StepKind::Inhale, 4_000)));
    assert_eq!(session.phase_remaining_ms(), Some(4_000));
    assert_eq!(session.session_remaining_ms(), None);
}

#[test]
fn empty_preset_is_completed_without_panicking() {
    let empty = breath::Preset {
        id: PresetId::Awake,
        default_name: "Empty",
        steps: [0, 0, 0, 0],
    };

    let session = Session::start(&empty, None);

    assert_eq!(session.status(), SessionStatus::Completed);
    assert_eq!(session.current_step(), None);
}

#[test]
fn phase_progress_reports_elapsed_fraction() {
    let mut session = Session::start(preset_by_id(PresetId::Awake), None);

    assert_eq!(session.phase_progress(), Some(0.0));
    session.advance(1_500);

    assert_eq!(session.phase_progress(), Some(0.25));
}

#[test]
fn countdown_session_waits_before_starting_breathing() {
    let mut session = Session::start_countdown(preset_by_id(PresetId::Awake), None, 3_000);

    assert_eq!(session.status(), SessionStatus::Countdown);
    assert_eq!(session.countdown_remaining_ms(), Some(3_000));
    assert_eq!(session.current_step(), None);

    session.advance(2_000);
    assert_eq!(session.status(), SessionStatus::Countdown);
    assert_eq!(session.countdown_remaining_ms(), Some(1_000));

    session.advance(1_000);
    assert_eq!(session.status(), SessionStatus::Running);
    assert_eq!(session.current_step(), Some((StepKind::Inhale, 6_000)));
}
