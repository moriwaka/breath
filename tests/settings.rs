use breath::{AudioMode, SessionLength, StepKind};

#[test]
fn default_session_length_is_five_minutes() {
    assert_eq!(SessionLength::default().as_option_ms(), Some(300_000));
}

#[test]
fn zero_minutes_means_an_unlimited_session() {
    assert_eq!(SessionLength::from_minutes(0).as_option_ms(), None);
}

#[test]
fn session_length_is_clamped_to_the_supported_range() {
    assert_eq!(SessionLength::from_minutes(61).minutes(), 60);
    assert_eq!(SessionLength::from_minutes(1).minutes(), 1);
}

#[test]
fn paul_is_the_default_guidance_voice() {
    assert_eq!(AudioMode::default(), AudioMode::Paul);
    assert_eq!(AudioMode::from_key("bell"), AudioMode::Bell);
    assert_eq!(AudioMode::from_key("unknown"), AudioMode::Paul);
}

#[test]
fn audio_modes_resolve_the_correct_phase_cues() {
    assert_eq!(
        AudioMode::Paul.asset_for_step(StepKind::Inhale),
        Some("paulinhale.mp3")
    );
    assert_eq!(
        AudioMode::Laura.asset_for_step(StepKind::HoldAfterInhale),
        Some("laurahold.mp3")
    );
    assert_eq!(
        AudioMode::Bell.asset_for_step(StepKind::Exhale),
        Some("cuebell2.mp3")
    );
    assert_eq!(AudioMode::Off.asset_for_step(StepKind::Exhale), None);
}

#[test]
fn only_enabled_audio_modes_play_the_completion_cue() {
    assert!(AudioMode::Paul.plays_completion_cue());
    assert!(AudioMode::Laura.plays_completion_cue());
    assert!(AudioMode::Bell.plays_completion_cue());
    assert!(!AudioMode::Off.plays_completion_cue());
}
