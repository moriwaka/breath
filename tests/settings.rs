use breath::{AudioMode, SessionLength};

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
