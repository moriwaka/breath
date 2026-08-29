//! Core domain logic for Breath.

/// A phase in one breathing cycle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StepKind {
    Inhale,
    HoldAfterInhale,
    Exhale,
    HoldAfterExhale,
}

/// The stable identifier for a built-in breathing preset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresetId {
    DeepCalm,
    Awake,
    Coherent,
    ExtendedExhale,
    Pranayama,
    Square,
    Ujjayi,
}

impl PresetId {
    pub const ALL: [Self; 7] = [
        Self::DeepCalm,
        Self::Awake,
        Self::Coherent,
        Self::ExtendedExhale,
        Self::Pranayama,
        Self::Square,
        Self::Ujjayi,
    ];
}

/// A fixed sequence of inhale, hold, exhale, and hold durations in milliseconds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Preset {
    pub id: PresetId,
    pub default_name: &'static str,
    pub steps: [u32; 4],
}

const DEEP_CALM: Preset = Preset {
    id: PresetId::DeepCalm,
    default_name: "4-7-8 Deep Calm",
    steps: [4_000, 7_000, 8_000, 0],
};
const AWAKE: Preset = Preset {
    id: PresetId::Awake,
    default_name: "Awake",
    steps: [6_000, 0, 2_000, 0],
};
const COHERENT: Preset = Preset {
    id: PresetId::Coherent,
    default_name: "Coherent",
    steps: [5_500, 0, 5_500, 0],
};
const EXTENDED_EXHALE: Preset = Preset {
    id: PresetId::ExtendedExhale,
    default_name: "Extended Exhale",
    steps: [4_000, 0, 6_000, 0],
};
const PRANAYAMA: Preset = Preset {
    id: PresetId::Pranayama,
    default_name: "Pranayama",
    steps: [7_000, 4_000, 8_000, 4_000],
};
const SQUARE: Preset = Preset {
    id: PresetId::Square,
    default_name: "Square",
    steps: [4_000, 4_000, 4_000, 4_000],
};
const UJJAYI: Preset = Preset {
    id: PresetId::Ujjayi,
    default_name: "Ujjayi",
    steps: [7_000, 0, 7_000, 0],
};

/// Returns a built-in preset by its stable identifier.
pub const fn preset_by_id(id: PresetId) -> &'static Preset {
    match id {
        PresetId::DeepCalm => &DEEP_CALM,
        PresetId::Awake => &AWAKE,
        PresetId::Coherent => &COHERENT,
        PresetId::ExtendedExhale => &EXTENDED_EXHALE,
        PresetId::Pranayama => &PRANAYAMA,
        PresetId::Square => &SQUARE,
        PresetId::Ujjayi => &UJJAYI,
    }
}

/// Returns the non-zero phases in the order a session presents them.
pub fn active_steps(preset: &Preset) -> Vec<(StepKind, u32)> {
    [
        (StepKind::Inhale, preset.steps[0]),
        (StepKind::HoldAfterInhale, preset.steps[1]),
        (StepKind::Exhale, preset.steps[2]),
        (StepKind::HoldAfterExhale, preset.steps[3]),
    ]
    .into_iter()
    .filter(|(_, duration)| *duration > 0)
    .collect()
}
