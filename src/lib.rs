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

/// The externally visible lifecycle of a guided session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionStatus {
    Running,
    Paused,
    Completed,
    Stopped,
}

/// A persisted duration selection. Zero minutes denotes no automatic end time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionLength(u8);

impl SessionLength {
    pub const DEFAULT_MINUTES: u8 = 5;
    pub const MAX_MINUTES: u8 = 60;

    pub fn from_minutes(minutes: u8) -> Self {
        Self(minutes.min(Self::MAX_MINUTES))
    }

    pub fn minutes(self) -> u8 {
        self.0
    }

    pub fn as_option_ms(self) -> Option<u32> {
        (self.0 != 0).then(|| u32::from(self.0) * 60_000)
    }
}

impl Default for SessionLength {
    fn default() -> Self {
        Self(Self::DEFAULT_MINUTES)
    }
}

/// The available phase-audio choices.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AudioMode {
    Paul,
    Laura,
    Bell,
    Off,
}

impl AudioMode {
    pub fn from_key(value: &str) -> Self {
        match value {
            "paul" => Self::Paul,
            "laura" => Self::Laura,
            "bell" => Self::Bell,
            "off" => Self::Off,
            _ => Self::default(),
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Self::Paul => "paul",
            Self::Laura => "laura",
            Self::Bell => "bell",
            Self::Off => "off",
        }
    }

    /// Returns the bundled audio asset that announces a breathing phase.
    /// `None` deliberately represents silent guidance.
    pub fn asset_for_step(self, step: StepKind) -> Option<&'static str> {
        match (self, step) {
            (Self::Paul, StepKind::Inhale) => Some("paulinhale.mp3"),
            (Self::Paul, StepKind::HoldAfterInhale | StepKind::HoldAfterExhale) => {
                Some("paulhold.mp3")
            }
            (Self::Paul, StepKind::Exhale) => Some("paulexhale.mp3"),
            (Self::Laura, StepKind::Inhale) => Some("laurainhale.mp3"),
            (Self::Laura, StepKind::HoldAfterInhale | StepKind::HoldAfterExhale) => {
                Some("laurahold.mp3")
            }
            (Self::Laura, StepKind::Exhale) => Some("lauraexhale.mp3"),
            (Self::Bell, StepKind::Inhale | StepKind::HoldAfterExhale) => Some("cuebell1.mp3"),
            (Self::Bell, StepKind::HoldAfterInhale | StepKind::Exhale) => Some("cuebell2.mp3"),
            (Self::Off, _) => None,
        }
    }
}

impl Default for AudioMode {
    fn default() -> Self {
        Self::Paul
    }
}

/// Time-driven state for one guided breathing session.
#[derive(Clone, Debug)]
pub struct Session {
    steps: Vec<(StepKind, u32)>,
    phase_index: usize,
    phase_elapsed_ms: u32,
    session_elapsed_ms: u32,
    session_limit_ms: Option<u32>,
    status: SessionStatus,
}

impl Session {
    /// Starts a session immediately. `None` denotes an unlimited session.
    pub fn start(preset: &Preset, session_limit_ms: Option<u32>) -> Self {
        Self {
            steps: active_steps(preset),
            phase_index: 0,
            phase_elapsed_ms: 0,
            session_elapsed_ms: 0,
            session_limit_ms,
            status: SessionStatus::Running,
        }
    }

    pub fn status(&self) -> SessionStatus {
        self.status
    }

    pub fn current_step(&self) -> Option<(StepKind, u32)> {
        (self.status == SessionStatus::Running || self.status == SessionStatus::Paused)
            .then(|| self.steps[self.phase_index])
    }

    pub fn phase_remaining_ms(&self) -> Option<u32> {
        self.current_step()
            .map(|(_, duration)| duration - self.phase_elapsed_ms)
    }

    pub fn session_remaining_ms(&self) -> Option<u32> {
        self.session_limit_ms
            .map(|limit| limit.saturating_sub(self.session_elapsed_ms))
    }

    pub fn pause(&mut self) {
        if self.status == SessionStatus::Running {
            self.status = SessionStatus::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.status == SessionStatus::Paused {
            self.status = SessionStatus::Running;
        }
    }

    pub fn stop(&mut self) {
        if self.status == SessionStatus::Running || self.status == SessionStatus::Paused {
            self.status = SessionStatus::Stopped;
        }
    }

    /// Advances the session by active elapsed time in milliseconds.
    pub fn advance(&mut self, elapsed_ms: u32) {
        if self.status != SessionStatus::Running {
            return;
        }

        let allowed_ms = self
            .session_limit_ms
            .map(|limit| {
                limit
                    .saturating_sub(self.session_elapsed_ms)
                    .min(elapsed_ms)
            })
            .unwrap_or(elapsed_ms);

        self.advance_phase(allowed_ms);
        self.session_elapsed_ms = self.session_elapsed_ms.saturating_add(allowed_ms);

        if self
            .session_limit_ms
            .is_some_and(|limit| self.session_elapsed_ms >= limit)
        {
            self.status = SessionStatus::Completed;
        }
    }

    fn advance_phase(&mut self, mut elapsed_ms: u32) {
        while elapsed_ms > 0 {
            let (_, duration_ms) = self.steps[self.phase_index];
            let remaining_ms = duration_ms - self.phase_elapsed_ms;

            if elapsed_ms < remaining_ms {
                self.phase_elapsed_ms += elapsed_ms;
                return;
            }

            elapsed_ms -= remaining_ms;
            self.phase_elapsed_ms = 0;
            self.phase_index = (self.phase_index + 1) % self.steps.len();
        }
    }
}
