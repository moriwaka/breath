use breath::{PresetId, StepKind, active_steps, preset_by_id};

#[test]
fn square_preset_exposes_all_four_breathing_phases() {
    let preset = preset_by_id(PresetId::Square);

    assert_eq!(preset.default_name, "Square");
    assert_eq!(preset.steps, [4_000, 4_000, 4_000, 4_000]);
    assert_eq!(
        active_steps(preset),
        vec![
            (StepKind::Inhale, 4_000),
            (StepKind::HoldAfterInhale, 4_000),
            (StepKind::Exhale, 4_000),
            (StepKind::HoldAfterExhale, 4_000),
        ]
    );
}

#[test]
fn zero_duration_hold_is_omitted_from_a_preset_session() {
    let preset = preset_by_id(PresetId::DeepCalm);

    assert_eq!(preset.steps, [4_000, 7_000, 8_000, 0]);
    assert_eq!(
        active_steps(preset),
        vec![
            (StepKind::Inhale, 4_000),
            (StepKind::HoldAfterInhale, 7_000),
            (StepKind::Exhale, 8_000),
        ]
    );
}

#[test]
fn every_breathly_preset_is_available() {
    assert_eq!(PresetId::ALL.len(), 7);
    assert_eq!(preset_by_id(PresetId::Awake).steps, [6_000, 0, 2_000, 0]);
    assert_eq!(preset_by_id(PresetId::Coherent).steps, [5_500, 0, 5_500, 0]);
    assert_eq!(
        preset_by_id(PresetId::ExtendedExhale).steps,
        [4_000, 0, 6_000, 0]
    );
    assert_eq!(
        preset_by_id(PresetId::Pranayama).steps,
        [7_000, 4_000, 8_000, 4_000]
    );
    assert_eq!(preset_by_id(PresetId::Ujjayi).steps, [7_000, 0, 7_000, 0]);
}
