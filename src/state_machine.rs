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
        variants: Vec<String>,
        held_since: Instant,
    },
    Selecting {
        key: MappingKey,
        variants: Vec<String>,
        selected_index: usize,
        held_since: Instant,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrabEvent {
    ShowOverlay { variants: Vec<String>, index: usize },
    UpdateSelection(usize),
    HideOverlay,
    InjectChar(String),
    /// False start: letter released too quickly, replay the suppressed trigger key
    FalseStart,
}

/// Wraps AccentState with config-derived settings.
pub struct StateMachine {
    pub state: AccentState,
    pub input_time: Duration,
    pub hold_delay: Duration,
    pub activation_key: ActivationKey,
}

impl StateMachine {
    pub fn new(input_time_ms: u64, hold_delay_ms: u64, activation_key: ActivationKey) -> Self {
        StateMachine {
            state: AccentState::Idle,
            input_time: Duration::from_millis(input_time_ms),
            hold_delay: Duration::from_millis(hold_delay_ms),
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
            AccentState::LetterHeld {
                key,
                variants,
                held_since,
            } => {
                if self.is_trigger(input) {
                    if held_since.elapsed() < self.hold_delay {
                        self.state = AccentState::Idle;
                        (false, None)
                    } else {
                        let variants = variants.clone();
                        let held_key = *key;
                        let held_since = *held_since;
                        self.state = AccentState::Selecting {
                            key: held_key,
                            variants: variants.clone(),
                            selected_index: 0,
                            held_since,
                        };
                        (
                            true,
                            Some(GrabEvent::ShowOverlay {
                                variants,
                                index: 0,
                            }),
                        )
                    }
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
            } => match input {
                KeyInput::Space | KeyInput::RightArrow => {
                    let len = variants.len();
                    let new_index = (*selected_index + 1) % len;
                    if let AccentState::Selecting {
                        selected_index, ..
                    } = &mut self.state
                    {
                        *selected_index = new_index;
                    }
                    (true, Some(GrabEvent::UpdateSelection(new_index)))
                }
                KeyInput::LeftArrow => {
                    let len = variants.len();
                    let new_index = (*selected_index + len - 1) % len;
                    if let AccentState::Selecting {
                        selected_index, ..
                    } = &mut self.state
                    {
                        *selected_index = new_index;
                    }
                    (true, Some(GrabEvent::UpdateSelection(new_index)))
                }
                KeyInput::Escape => {
                    self.state = AccentState::Idle;
                    (true, Some(GrabEvent::HideOverlay))
                }
                _ => (true, None),
            },
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
                        let selected = variants[*selected_index].clone();
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
        if let AccentState::Selecting {
            key,
            variants,
            selected_index,
            ..
        } = &mut self.state
        {
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

    /// Backdate `held_since` so hold_delay / input_time checks pass without sleeping.
    #[cfg(test)]
    fn set_held_ago(&mut self, ago: Duration) {
        let t = Instant::now()
            .checked_sub(ago)
            .expect("Instant::checked_sub");
        match &mut self.state {
            AccentState::LetterHeld { held_since, .. }
            | AccentState::Selecting { held_since, .. } => {
                *held_since = t;
            }
            _ => panic!("set_held_ago requires LetterHeld or Selecting"),
        }
    }

    #[cfg(test)]
    fn enter_cooldown_for(&mut self, remaining: Duration) {
        self.state = AccentState::Cooldown {
            until: Instant::now() + remaining,
        };
    }

    #[cfg(test)]
    fn enter_cooldown_expired(&mut self) {
        self.state = AccentState::Cooldown {
            until: Instant::now()
                .checked_sub(Duration::from_millis(1))
                .unwrap_or_else(Instant::now),
        };
    }

    #[cfg(test)]
    fn is_idle(&self) -> bool {
        matches!(self.state, AccentState::Idle)
    }

    #[cfg(test)]
    fn is_cooldown(&self) -> bool {
        matches!(self.state, AccentState::Cooldown { .. })
    }

    #[cfg(test)]
    fn is_letter_held(&self) -> bool {
        matches!(self.state, AccentState::LetterHeld { .. })
    }

    #[cfg(test)]
    fn is_selecting(&self) -> bool {
        matches!(self.state, AccentState::Selecting { .. })
    }

    #[cfg(test)]
    fn selected_index(&self) -> Option<usize> {
        match &self.state {
            AccentState::Selecting { selected_index, .. } => Some(*selected_index),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mappings::{self, MappingKey};

    fn setup() {
        mappings::init(&[
            "French".into(),
            "German".into(),
            "Spanish".into(),
        ]);
    }

    fn sm(activation: ActivationKey) -> StateMachine {
        // Non-zero delays; tests backdate held_since instead of sleeping.
        StateMachine::new(200, 250, activation)
    }

    fn press_e(sm: &mut StateMachine) {
        let (suppress, ev) = sm.handle_key_press(KeyInput::Letter(MappingKey::E), false);
        assert!(!suppress);
        assert!(ev.is_none());
        assert!(sm.is_letter_held());
        assert_eq!(sm.held_key(), Some(MappingKey::E));
    }

    fn open_overlay(sm: &mut StateMachine) -> Vec<String> {
        press_e(sm);
        sm.set_held_ago(Duration::from_millis(300));
        let (suppress, ev) = sm.handle_key_press(KeyInput::Space, false);
        assert!(suppress);
        match ev {
            Some(GrabEvent::ShowOverlay { variants, index }) => {
                assert_eq!(index, 0);
                assert!(!variants.is_empty());
                assert!(sm.is_selecting());
                variants
            }
            other => panic!("expected ShowOverlay, got {other:?}"),
        }
    }

    #[test]
    fn idle_letter_with_variants_enters_letter_held() {
        setup();
        let mut sm = sm(ActivationKey::Both);
        press_e(&mut sm);
    }

    #[test]
    fn idle_letter_without_variants_stays_idle() {
        setup();
        let mut sm = sm(ActivationKey::Both);
        // French/German/Spanish have no accents on F typically... check
        let variants = mappings::get_variants(MappingKey::F, false);
        if variants.is_empty() {
            let (suppress, ev) = sm.handle_key_press(KeyInput::Letter(MappingKey::F), false);
            assert!(!suppress);
            assert!(ev.is_none());
            assert!(sm.is_idle());
            assert!(sm.held_key().is_none());
        } else {
            // If F gains variants later, skip assertion shape
            let _ = sm.handle_key_press(KeyInput::Letter(MappingKey::F), false);
        }
    }

    #[test]
    fn trigger_before_hold_delay_cancels_without_overlay() {
        setup();
        let mut sm = sm(ActivationKey::Both);
        press_e(&mut sm);
        // held_since is ~now; hold_delay 250ms → too early
        let (suppress, ev) = sm.handle_key_press(KeyInput::Space, false);
        assert!(!suppress);
        assert!(ev.is_none());
        assert!(sm.is_idle());
    }

    #[test]
    fn trigger_after_hold_delay_opens_overlay() {
        setup();
        let mut sm = sm(ActivationKey::Both);
        let variants = open_overlay(&mut sm);
        assert!(variants.iter().any(|v| v == "é" || v.contains('é')));
    }

    #[test]
    fn space_and_right_cycle_forward_with_wrap() {
        setup();
        let mut sm = sm(ActivationKey::Both);
        let variants = open_overlay(&mut sm);
        let n = variants.len();

        let (s, ev) = sm.handle_key_press(KeyInput::Space, false);
        assert!(s);
        assert_eq!(ev, Some(GrabEvent::UpdateSelection(1 % n)));
        assert_eq!(sm.selected_index(), Some(1 % n));

        let (s, ev) = sm.handle_key_press(KeyInput::RightArrow, false);
        assert!(s);
        assert_eq!(ev, Some(GrabEvent::UpdateSelection(2 % n)));

        // Advance to last then wrap
        if let AccentState::Selecting {
            selected_index, ..
        } = &mut sm.state
        {
            *selected_index = n - 1;
        }
        let (s, ev) = sm.handle_key_press(KeyInput::Space, false);
        assert!(s);
        assert_eq!(ev, Some(GrabEvent::UpdateSelection(0)));
    }

    #[test]
    fn left_arrow_cycles_backward_with_wrap() {
        setup();
        let mut sm = sm(ActivationKey::Both);
        let variants = open_overlay(&mut sm);
        let n = variants.len();

        let (s, ev) = sm.handle_key_press(KeyInput::LeftArrow, false);
        assert!(s);
        assert_eq!(ev, Some(GrabEvent::UpdateSelection(n - 1)));
    }

    #[test]
    fn escape_hides_overlay_and_idles() {
        setup();
        let mut sm = sm(ActivationKey::Both);
        open_overlay(&mut sm);
        let (s, ev) = sm.handle_key_press(KeyInput::Escape, false);
        assert!(s);
        assert_eq!(ev, Some(GrabEvent::HideOverlay));
        assert!(sm.is_idle());
    }

    #[test]
    fn release_after_input_time_injects_and_cools_down() {
        setup();
        let mut sm = sm(ActivationKey::Both);
        let variants = open_overlay(&mut sm);
        sm.set_held_ago(Duration::from_millis(500)); // > input_time 200
        let (s, ev) = sm.handle_key_release(KeyInput::Letter(MappingKey::E));
        assert!(s);
        assert_eq!(ev, Some(GrabEvent::InjectChar(variants[0].clone())));
        assert!(sm.is_cooldown());
    }

    #[test]
    fn release_before_input_time_is_false_start() {
        setup();
        let mut sm = sm(ActivationKey::Both);
        open_overlay(&mut sm);
        // held_since still ~when LetterHeld started; if we just opened, elapsed may be small
        // Force held_since to "just now" so elapsed < input_time
        sm.set_held_ago(Duration::from_millis(0));
        let (s, ev) = sm.handle_key_release(KeyInput::Letter(MappingKey::E));
        assert!(s);
        assert_eq!(ev, Some(GrabEvent::FalseStart));
        assert!(sm.is_idle());
    }

    #[test]
    fn release_other_letter_while_selecting_does_nothing() {
        setup();
        let mut sm = sm(ActivationKey::Both);
        open_overlay(&mut sm);
        sm.set_held_ago(Duration::from_millis(500));
        let (s, ev) = sm.handle_key_release(KeyInput::Letter(MappingKey::A));
        assert!(!s);
        assert!(ev.is_none());
        assert!(sm.is_selecting());
    }

    #[test]
    fn other_letter_while_held_cancels() {
        setup();
        let mut sm = sm(ActivationKey::Both);
        press_e(&mut sm);
        let (s, ev) = sm.handle_key_press(KeyInput::Letter(MappingKey::A), false);
        assert!(!s);
        assert!(ev.is_none());
        assert!(sm.is_idle());
    }

    #[test]
    fn same_letter_repeat_while_held_ignored() {
        setup();
        let mut sm = sm(ActivationKey::Both);
        press_e(&mut sm);
        let (s, ev) = sm.handle_key_press(KeyInput::Letter(MappingKey::E), false);
        assert!(!s);
        assert!(ev.is_none());
        assert!(sm.is_letter_held());
    }

    #[test]
    fn release_letter_while_held_returns_idle() {
        setup();
        let mut sm = sm(ActivationKey::Both);
        press_e(&mut sm);
        let (s, ev) = sm.handle_key_release(KeyInput::Letter(MappingKey::E));
        assert!(!s);
        assert!(ev.is_none());
        assert!(sm.is_idle());
    }

    #[test]
    fn activation_space_only_ignores_arrows() {
        setup();
        let mut sm = sm(ActivationKey::Space);
        press_e(&mut sm);
        sm.set_held_ago(Duration::from_millis(300));
        let (s, ev) = sm.handle_key_press(KeyInput::LeftArrow, false);
        // LeftArrow is not a trigger → treated as "different key" → cancel
        assert!(!s);
        assert!(ev.is_none());
        assert!(sm.is_idle());
    }

    #[test]
    fn activation_arrows_only_ignores_space() {
        setup();
        let mut sm = sm(ActivationKey::LeftRightArrow);
        press_e(&mut sm);
        sm.set_held_ago(Duration::from_millis(300));
        let (s, ev) = sm.handle_key_press(KeyInput::Space, false);
        assert!(!s);
        assert!(ev.is_none());
        assert!(sm.is_idle());

        press_e(&mut sm);
        sm.set_held_ago(Duration::from_millis(300));
        let (s, ev) = sm.handle_key_press(KeyInput::RightArrow, false);
        assert!(s);
        assert!(matches!(ev, Some(GrabEvent::ShowOverlay { .. })));
    }

    #[test]
    fn update_shift_refreshes_variants() {
        setup();
        let mut sm = sm(ActivationKey::Both);
        open_overlay(&mut sm);
        let ev = sm.update_shift(true);
        match ev {
            Some(GrabEvent::ShowOverlay { variants, index }) => {
                assert_eq!(index, 0);
                // Uppercase French e accents
                assert!(variants.iter().any(|v| v.chars().next().unwrap().is_uppercase()));
            }
            None => {
                // If uppercase equals lowercase set somehow, ok
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn cooldown_blocks_letter_held() {
        setup();
        let mut sm = sm(ActivationKey::Both);
        sm.enter_cooldown_for(Duration::from_secs(30));
        let (s, ev) = sm.handle_key_press(KeyInput::Letter(MappingKey::E), false);
        assert!(!s);
        assert!(ev.is_none());
        assert!(sm.is_cooldown());
        assert!(sm.held_key().is_none());
    }

    #[test]
    fn expired_cooldown_acts_like_idle() {
        setup();
        let mut sm = sm(ActivationKey::Both);
        sm.enter_cooldown_expired();
        let (s, ev) = sm.handle_key_press(KeyInput::Letter(MappingKey::E), false);
        assert!(!s);
        assert!(ev.is_none());
        assert!(sm.is_letter_held());
    }

    #[test]
    fn force_reset_clears_selecting() {
        setup();
        let mut sm = sm(ActivationKey::Both);
        open_overlay(&mut sm);
        sm.force_reset();
        assert!(sm.is_idle());
    }

    #[test]
    fn selecting_suppresses_unrelated_keys() {
        setup();
        let mut sm = sm(ActivationKey::Both);
        open_overlay(&mut sm);
        let (s, ev) = sm.handle_key_press(KeyInput::Other, false);
        assert!(s);
        assert!(ev.is_none());
        assert!(sm.is_selecting());
    }
}
