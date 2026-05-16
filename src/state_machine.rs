use std::time::{Duration, Instant};

use crate::mappings::MappingKey;

/// After selecting an accent, ignore accent triggers for this long
const COOLDOWN_DURATION: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyInput {
    Letter(MappingKey),
    Space,
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
    },
    Selecting {
        key: MappingKey,
        variants: Vec<char>,
        selected_index: usize,
    },
}

#[derive(Debug, Clone)]
pub enum GrabEvent {
    ShowOverlay { variants: Vec<char>, index: usize },
    UpdateSelection(usize),
    HideOverlay,
    InjectChar(char),
}

impl AccentState {
    pub fn new() -> Self {
        AccentState::Idle
    }

    fn try_enter_letter_held(&mut self, mk: MappingKey, shift_held: bool) {
        let variants = crate::mappings::get_variants(mk, shift_held);
        if !variants.is_empty() {
            *self = AccentState::LetterHeld { key: mk, variants };
        }
    }

    pub fn handle_key_press(
        &mut self,
        input: KeyInput,
        shift_held: bool,
    ) -> (bool, Option<GrabEvent>) {
        match self {
            AccentState::Idle => {
                if let KeyInput::Letter(mk) = input {
                    self.try_enter_letter_held(mk, shift_held);
                }
                (false, None)
            }
            AccentState::Cooldown { until } => {
                if Instant::now() >= *until {
                    // Cooldown expired → act like Idle
                    *self = AccentState::Idle;
                    if let KeyInput::Letter(mk) = input {
                        self.try_enter_letter_held(mk, shift_held);
                    }
                }
                // During cooldown, pass everything through
                (false, None)
            }
            AccentState::LetterHeld { key, variants, .. } => {
                if input == KeyInput::Space {
                    // Activate accent selection immediately
                    let variants = variants.clone();
                    let held_key = *key;
                    *self = AccentState::Selecting {
                        key: held_key,
                        variants: variants.clone(),
                        selected_index: 0,
                    };
                    (true, Some(GrabEvent::ShowOverlay { variants, index: 0 }))
                } else if input == KeyInput::Letter(*key) {
                    // Same key repeat → ignore
                    (false, None)
                } else {
                    // Different key pressed → cancel
                    *self = AccentState::Idle;
                    (false, None)
                }
            }
            AccentState::Selecting {
                variants,
                selected_index,
                ..
            } => {
                if input == KeyInput::Space {
                    let new_index = (*selected_index + 1) % variants.len();
                    *selected_index = new_index;
                    (true, Some(GrabEvent::UpdateSelection(new_index)))
                } else if input == KeyInput::Escape {
                    *self = AccentState::Idle;
                    (true, Some(GrabEvent::HideOverlay))
                } else {
                    (true, None)
                }
            }
        }
    }

    pub fn handle_key_release(&mut self, input: KeyInput) -> (bool, Option<GrabEvent>) {
        match self {
            AccentState::LetterHeld { key, .. } => {
                if let KeyInput::Letter(mk) = input {
                    if mk == *key {
                        *self = AccentState::Idle;
                    }
                }
                (false, None)
            }
            AccentState::Selecting {
                key,
                variants,
                selected_index,
                ..
            } => {
                if let KeyInput::Letter(mk) = input {
                    if mk == *key {
                        let selected = variants[*selected_index];
                        // Enter cooldown so next letter+space is normal typing
                        *self = AccentState::Cooldown {
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
}
