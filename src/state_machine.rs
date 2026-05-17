use std::time::{Duration, Instant};

use crate::config::ActivationKey;
use crate::mappings::MappingKey;

/// After selecting an accent, ignore accent triggers for this long
const COOLDOWN_DURATION: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyInput {
    Letter(MappingKey),
    Space,
    LeftArrow,
    RightArrow,
    Escape,
    Other,
}

#[derive(Clone)]
pub enum AccentState {
    Idle,
    /// Brief cooldown after accent injection — behaves like Idle but won't
    /// enter LetterHeld, so the next letter+space is normal typing.
    Cooldown {
        until: Instant,
    },
    LetterHeld {
        key: MappingKey,
        variants: Vec<char>,
        held_since: Instant,
    },
    Selecting {
        key: MappingKey,
        variants: Vec<char>,
        selected_index: usize,
        held_since: Instant,
    },
}

#[derive(Debug, Clone)]
pub enum GrabEvent {
    ShowOverlay { variants: Vec<char>, index: usize },
    UpdateSelection(usize),
    HideOverlay,
    InjectChar(char),
    /// False start: letter released too quickly, replay the suppressed trigger key
    FalseStart,
}

/// Wraps AccentState with config-derived settings.
pub struct StateMachine {
    pub state: AccentState,
    pub input_time: Duration,
    pub activation_key: ActivationKey,
}

impl StateMachine {
    pub fn new(input_time_ms: u64, activation_key: ActivationKey) -> Self {
        StateMachine {
            state: AccentState::Idle,
            input_time: Duration::from_millis(input_time_ms),
            activation_key,
        }
    }

    /// Check if the given input is an allowed trigger for entering Selecting state.
    fn is_trigger(&self, input: KeyInput) -> bool {
        match self.activation_key {
            ActivationKey::Space => input == KeyInput::Space,
            ActivationKey::LeftRightArrow => {
                matches!(input, KeyInput::LeftArrow | KeyInput::RightArrow)
            }
            ActivationKey::Both => {
                matches!(input, KeyInput::Space | KeyInput::LeftArrow | KeyInput::RightArrow)
            }
        }
    }

    fn try_enter_letter_held(&mut self, mk: MappingKey, shift_held: bool) {
        let variants = crate::mappings::get_variants(mk, shift_held);
        if !variants.is_empty() {
            self.state = AccentState::LetterHeld {
                key: mk,
                variants,
                held_since: Instant::now(),
            };
        }
    }

    pub fn handle_key_press(
        &mut self,
        input: KeyInput,
        shift_held: bool,
    ) -> (bool, Option<GrabEvent>) {
        match &self.state {
            AccentState::Idle => {
                if let KeyInput::Letter(mk) = input {
                    self.try_enter_letter_held(mk, shift_held);
                }
                (false, None)
            }
            AccentState::Cooldown { until } => {
                if Instant::now() >= *until {
                    // Cooldown expired → act like Idle
                    self.state = AccentState::Idle;
                    if let KeyInput::Letter(mk) = input {
                        self.try_enter_letter_held(mk, shift_held);
                    }
                }
                // During cooldown, pass everything through
                (false, None)
            }
            AccentState::LetterHeld { key, variants, held_since } => {
                if self.is_trigger(input) {
                    // Activate accent selection immediately
                    let variants = variants.clone();
                    let held_key = *key;
                    let held_since = *held_since;
                    self.state = AccentState::Selecting {
                        key: held_key,
                        variants: variants.clone(),
                        selected_index: 0,
                        held_since,
                    };
                    (true, Some(GrabEvent::ShowOverlay { variants, index: 0 }))
                } else if input == KeyInput::Letter(*key) {
                    // Same key repeat → ignore
                    (false, None)
                } else {
                    // Different key pressed → cancel
                    self.state = AccentState::Idle;
                    (false, None)
                }
            }
            AccentState::Selecting {
                variants,
                selected_index,
                ..
            } => {
                match input {
                    KeyInput::Space | KeyInput::RightArrow => {
                        let len = variants.len();
                        let new_index = (*selected_index + 1) % len;
                        if let AccentState::Selecting { selected_index, .. } = &mut self.state {
                            *selected_index = new_index;
                        }
                        (true, Some(GrabEvent::UpdateSelection(new_index)))
                    }
                    KeyInput::LeftArrow => {
                        let len = variants.len();
                        let new_index = (*selected_index + len - 1) % len;
                        if let AccentState::Selecting { selected_index, .. } = &mut self.state {
                            *selected_index = new_index;
                        }
                        (true, Some(GrabEvent::UpdateSelection(new_index)))
                    }
                    KeyInput::Escape => {
                        self.state = AccentState::Idle;
                        (true, Some(GrabEvent::HideOverlay))
                    }
                    _ => (true, None),
                }
            }
        }
    }

    pub fn handle_key_release(&mut self, input: KeyInput) -> (bool, Option<GrabEvent>) {
        match &self.state {
            AccentState::LetterHeld { key, .. } => {
                if let KeyInput::Letter(mk) = input {
                    if mk == *key {
                        self.state = AccentState::Idle;
                    }
                }
                (false, None)
            }
            AccentState::Selecting {
                key,
                variants,
                selected_index,
                held_since,
                ..
            } => {
                if let KeyInput::Letter(mk) = input {
                    if mk == *key {
                        let elapsed = held_since.elapsed();
                        if elapsed < self.input_time {
                            // False start: letter released too quickly
                            self.state = AccentState::Idle;
                            return (true, Some(GrabEvent::FalseStart));
                        }
                        let selected = variants[*selected_index];
                        // Enter cooldown so next letter+space is normal typing
                        self.state = AccentState::Cooldown {
                            until: Instant::now() + COOLDOWN_DURATION,
                        };
                        return (true, Some(GrabEvent::InjectChar(selected)));
                    }
                }
                (false, None)
            }
            _ => (false, None),
        }
    }

    /// Called when shift state changes. If in Selecting, refreshes variants.
    pub fn update_shift(&mut self, shift_held: bool) -> Option<GrabEvent> {
        if let AccentState::Selecting { key, variants, selected_index, .. } = &mut self.state {
            let new_variants = crate::mappings::get_variants(*key, shift_held);
            if new_variants != *variants {
                *variants = new_variants.clone();
                if *selected_index >= variants.len() {
                    *selected_index = 0;
                }
                return Some(GrabEvent::ShowOverlay {
                    variants: variants.clone(),
                    index: *selected_index,
                });
            }
        }
        None
    }

    /// Peek at the currently held key (if in LetterHeld state).
    pub fn held_key(&self) -> Option<MappingKey> {
        match &self.state {
            AccentState::LetterHeld { key, .. } => Some(*key),
            _ => None,
        }
    }

    /// Force reset to Idle (used when physical key verification fails).
    pub fn force_reset(&mut self) {
        self.state = AccentState::Idle;
    }
}
