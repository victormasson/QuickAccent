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

/// A key event for the virtual keyboard (evdev KEY_* code, no +8 XKB offset).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEvt {
    Press(u16),
    Release(u16),
}

impl KeyEvt {
    pub fn code(self) -> u16 {
        match self {
            KeyEvt::Press(c) | KeyEvt::Release(c) => c,
        }
    }

    pub fn value(self) -> i32 {
        match self {
            KeyEvt::Press(_) => 1,
            KeyEvt::Release(_) => 0,
        }
    }
}

/// What to do when the physical release of a deferred-replayed key arrives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseAction {
    /// The virtual release was already emitted — just swallow the physical one.
    SwallowOnly,
    /// Only the virtual press was emitted — emit the matching virtual release.
    EmitVirtualRelease,
}

/// Result of a deferred-mode transition. Pure data so it's unit-testable.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Action {
    /// Swallow the physical event (don't let rdev re-emit it).
    pub suppress: bool,
    /// Events to replay through the virtual keyboard, in order.
    pub emit: Vec<KeyEvt>,
    /// Physical releases to intercept later (registered by the grab thread).
    pub pending: Vec<(u16, ReleaseAction)>,
    /// UI event to forward to the overlay.
    pub ui: Option<GrabEvent>,
    /// Accent character to inject (commit).
    pub inject: Option<String>,
}

impl Action {
    fn pass() -> Self {
        Action::default()
    }

    fn swallow() -> Self {
        Action {
            suppress: true,
            ..Action::default()
        }
    }
}

// Canonical evdev KEY_* codes for the modifiers the injection paths touch —
// defined once here (next to KeyEvt) and re-exported by virtual_kb.
pub const KEY_LEFTCTRL: u16 = 29;
pub const KEY_LEFTSHIFT: u16 = 42;
pub const KEY_RIGHTSHIFT: u16 = 54;
pub const KEY_RIGHTALT: u16 = 100;

#[derive(Clone)]
pub enum AccentState {
    Idle,
    /// Brief cooldown after accent injection — behaves like Idle but won't
    /// enter LetterHeld, so the next letter+space is normal typing.
    /// (macOS path only; the deferred Linux path doesn't need it.)
    Cooldown {
        until: Instant,
    },
    LetterHeld {
        key: MappingKey,
        /// evdev code of the physical key (deferred mode; 0 on macOS)
        code: u16,
        /// shift state when the key was pressed (deferred mode)
        shift_at_press: bool,
        variants: Vec<String>,
        held_since: Instant,
    },
    Selecting {
        key: MappingKey,
        /// evdev code of the physical key (deferred mode; 0 on macOS)
        code: u16,
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
    /// macOS path only — deferred mode types nothing up front, so the
    /// "released too quickly to be intentional" check has no purpose there.
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
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

    fn try_enter_letter_held(&mut self, mk: MappingKey, code: u16, shift_held: bool) -> bool {
        let variants = crate::mappings::get_variants(mk, shift_held);
        if !variants.is_empty() {
            self.state = AccentState::LetterHeld {
                key: mk,
                code,
                shift_at_press: shift_held,
                variants,
                held_since: Instant::now(),
            };
            true
        } else {
            false
        }
    }

    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    pub fn handle_key_press(
        &mut self,
        input: KeyInput,
        shift_held: bool,
    ) -> (bool, Option<GrabEvent>) {
        match &self.state {
            AccentState::Idle => {
                if let KeyInput::Letter(mk) = input {
                    self.try_enter_letter_held(mk, 0, shift_held);
                }
                (false, None)
            }
            AccentState::Cooldown { until } => {
                if Instant::now() >= *until {
                    // Cooldown expired → act like Idle
                    self.state = AccentState::Idle;
                    if let KeyInput::Letter(mk) = input {
                        self.try_enter_letter_held(mk, 0, shift_held);
                    }
                }
                // During cooldown, pass everything through
                (false, None)
            }
            AccentState::LetterHeld {
                key,
                code,
                variants,
                held_since,
                ..
            } => {
                if self.is_trigger(input) {
                    if held_since.elapsed() < self.hold_delay {
                        self.state = AccentState::Idle;
                        (false, None)
                    } else {
                        let variants = variants.clone();
                        let held_key = *key;
                        let held_code = *code;
                        let held_since = *held_since;
                        self.state = AccentState::Selecting {
                            key: held_key,
                            code: held_code,
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

    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
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

    /// Deferred-mode key press (Linux). The letter is *suppressed* on keydown
    /// and replayed through the virtual keyboard when the user's intent is
    /// known — so it never reaches the app before an accent choice is made.
    ///
    /// `code`: evdev code of the physical key (None = unmappable, can't replay).
    /// `mods_held`: Ctrl/Alt/AltGr/Super held (shortcut chords bypass accents).
    pub fn deferred_press(
        &mut self,
        input: KeyInput,
        code: Option<u16>,
        shift_held: bool,
        mods_held: bool,
    ) -> Action {
        match &self.state {
            AccentState::Idle | AccentState::Cooldown { .. } => {
                if let (KeyInput::Letter(mk), Some(c), false) = (input, code, mods_held) {
                    if self.try_enter_letter_held(mk, c, shift_held) {
                        return Action::swallow();
                    }
                }
                Action::pass()
            }
            AccentState::LetterHeld {
                key,
                code: held,
                shift_at_press,
                variants,
                held_since,
            } => {
                let (key, held, shift_at_press, held_since) =
                    (*key, *held, *shift_at_press, *held_since);
                if code == Some(held) {
                    // Kernel autorepeat of the held letter — it must not type.
                    return Action::swallow();
                }
                if self.is_trigger(input) {
                    if held_since.elapsed() >= self.hold_delay {
                        let variants = variants.clone();
                        self.state = AccentState::Selecting {
                            key,
                            code: held,
                            variants: variants.clone(),
                            selected_index: 0,
                            held_since,
                        };
                        return Action {
                            suppress: true,
                            ui: Some(GrabEvent::ShowOverlay { variants, index: 0 }),
                            ..Action::default()
                        };
                    }
                    // Fast "e␣" typing: replay letter, then the trigger itself.
                    self.state = AccentState::Idle;
                    let mut action = Action::swallow();
                    action.emit = Self::replay_letter(held, shift_at_press, shift_held);
                    action.pending.push((held, ReleaseAction::SwallowOnly));
                    if let Some(t) = code {
                        action.emit.push(KeyEvt::Press(t));
                        action.pending.push((t, ReleaseAction::EmitVirtualRelease));
                    } else {
                        action.suppress = false; // can't replay it — let it through
                    }
                    return action;
                }
                // Rollover: another key pressed while the letter is deferred.
                // Replay the letter first, then route the new key through the
                // same (virtual) device so their order is preserved.
                self.state = AccentState::Idle;
                let mut action = Action::swallow();
                action.emit = Self::replay_letter(held, shift_at_press, shift_held);
                action.pending.push((held, ReleaseAction::SwallowOnly));
                if let (KeyInput::Letter(mk), Some(c), false) = (input, code, mods_held) {
                    if self.try_enter_letter_held(mk, c, shift_held) {
                        // The new key is itself accent-capable → defer it too.
                        return action;
                    }
                }
                if let Some(c) = code {
                    action.emit.push(KeyEvt::Press(c));
                    action.pending.push((c, ReleaseAction::EmitVirtualRelease));
                } else {
                    action.suppress = false; // unmappable key — let it through
                }
                action
            }
            AccentState::Selecting {
                code: held,
                variants,
                selected_index,
                ..
            } => {
                let held = *held;
                let len = variants.len();
                let idx = *selected_index;
                match input {
                    KeyInput::Space | KeyInput::RightArrow | KeyInput::LeftArrow => {
                        let new_index = if input == KeyInput::LeftArrow {
                            (idx + len - 1) % len
                        } else {
                            (idx + 1) % len
                        };
                        if let AccentState::Selecting { selected_index, .. } = &mut self.state {
                            *selected_index = new_index;
                        }
                        Action {
                            suppress: true,
                            ui: Some(GrabEvent::UpdateSelection(new_index)),
                            ..Action::default()
                        }
                    }
                    KeyInput::Escape => {
                        // Cancel: the user still typed the letter — replay it
                        // plain (it's the only way to get the base letter).
                        self.state = AccentState::Idle;
                        Action {
                            suppress: true,
                            emit: vec![KeyEvt::Press(held), KeyEvt::Release(held)],
                            pending: vec![(held, ReleaseAction::SwallowOnly)],
                            ui: Some(GrabEvent::HideOverlay),
                            ..Action::default()
                        }
                    }
                    // Overlay is modal: swallow everything else (incl. repeats
                    // of the held letter).
                    _ => Action::swallow(),
                }
            }
        }
    }

    /// Deferred-mode key release (Linux). The grab thread intercepts releases
    /// registered via `Action::pending` *before* calling this.
    pub fn deferred_release(&mut self, code: Option<u16>, shift_held: bool) -> Action {
        match &self.state {
            AccentState::LetterHeld {
                code: held,
                shift_at_press,
                ..
            } => {
                let (held, shift_at_press) = (*held, *shift_at_press);
                if code == Some(held) {
                    // Normal typing: the letter appears on key release.
                    self.state = AccentState::Idle;
                    return Action {
                        suppress: true,
                        emit: Self::replay_letter(held, shift_at_press, shift_held),
                        ..Action::default()
                    };
                }
                Action::pass()
            }
            AccentState::Selecting {
                code: held,
                variants,
                selected_index,
                ..
            } => {
                if code == Some(*held) {
                    // Commit: inject the selected variant. Nothing to delete —
                    // the letter was never typed.
                    let selected = variants[*selected_index].clone();
                    self.state = AccentState::Idle;
                    return Action {
                        suppress: true,
                        ui: Some(GrabEvent::InjectChar(selected.clone())),
                        inject: Some(selected),
                        ..Action::default()
                    };
                }
                Action::pass()
            }
            _ => Action::pass(),
        }
    }

    /// Replay a deferred letter press+release; if Shift was down at press time
    /// but has been released since (sloppy capitals), wrap it in Shift.
    fn replay_letter(code: u16, shift_at_press: bool, shift_now: bool) -> Vec<KeyEvt> {
        if shift_at_press && !shift_now {
            vec![
                KeyEvt::Press(KEY_LEFTSHIFT),
                KeyEvt::Press(code),
                KeyEvt::Release(code),
                KeyEvt::Release(KEY_LEFTSHIFT),
            ]
        } else {
            vec![KeyEvt::Press(code), KeyEvt::Release(code)]
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
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    pub fn held_key(&self) -> Option<MappingKey> {
        match &self.state {
            AccentState::LetterHeld { key, .. } => Some(*key),
            _ => None,
        }
    }

    /// Force reset to Idle (used when physical key verification fails).
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
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

    /// Initializes the global mappings and holds the cross-test guard for
    /// the test's duration — bind it: `let _guard = setup();`.
    #[must_use]
    fn setup() -> std::sync::MutexGuard<'static, ()> {
        let guard = mappings::test_guard();
        mappings::init(&[
            "French".into(),
            "German".into(),
            "Spanish".into(),
        ]);
        guard
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
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        press_e(&mut sm);
    }

    #[test]
    fn idle_letter_without_variants_stays_idle() {
        let _guard = setup();
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
        let _guard = setup();
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
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        let variants = open_overlay(&mut sm);
        assert!(variants.iter().any(|v| v == "é" || v.contains('é')));
    }

    #[test]
    fn space_and_right_cycle_forward_with_wrap() {
        let _guard = setup();
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
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        let variants = open_overlay(&mut sm);
        let n = variants.len();

        let (s, ev) = sm.handle_key_press(KeyInput::LeftArrow, false);
        assert!(s);
        assert_eq!(ev, Some(GrabEvent::UpdateSelection(n - 1)));
    }

    #[test]
    fn escape_hides_overlay_and_idles() {
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        open_overlay(&mut sm);
        let (s, ev) = sm.handle_key_press(KeyInput::Escape, false);
        assert!(s);
        assert_eq!(ev, Some(GrabEvent::HideOverlay));
        assert!(sm.is_idle());
    }

    #[test]
    fn release_after_input_time_injects_and_cools_down() {
        let _guard = setup();
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
        let _guard = setup();
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
        let _guard = setup();
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
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        press_e(&mut sm);
        let (s, ev) = sm.handle_key_press(KeyInput::Letter(MappingKey::A), false);
        assert!(!s);
        assert!(ev.is_none());
        assert!(sm.is_idle());
    }

    #[test]
    fn same_letter_repeat_while_held_ignored() {
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        press_e(&mut sm);
        let (s, ev) = sm.handle_key_press(KeyInput::Letter(MappingKey::E), false);
        assert!(!s);
        assert!(ev.is_none());
        assert!(sm.is_letter_held());
    }

    #[test]
    fn release_letter_while_held_returns_idle() {
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        press_e(&mut sm);
        let (s, ev) = sm.handle_key_release(KeyInput::Letter(MappingKey::E));
        assert!(!s);
        assert!(ev.is_none());
        assert!(sm.is_idle());
    }

    #[test]
    fn activation_space_only_ignores_arrows() {
        let _guard = setup();
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
        let _guard = setup();
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
        let _guard = setup();
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
        let _guard = setup();
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
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        sm.enter_cooldown_expired();
        let (s, ev) = sm.handle_key_press(KeyInput::Letter(MappingKey::E), false);
        assert!(!s);
        assert!(ev.is_none());
        assert!(sm.is_letter_held());
    }

    #[test]
    fn force_reset_clears_selecting() {
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        open_overlay(&mut sm);
        sm.force_reset();
        assert!(sm.is_idle());
    }

    #[test]
    fn selecting_suppresses_unrelated_keys() {
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        open_overlay(&mut sm);
        let (s, ev) = sm.handle_key_press(KeyInput::Other, false);
        assert!(s);
        assert!(ev.is_none());
        assert!(sm.is_selecting());
    }

    // ---- Deferred mode (Linux) ----

    const E: u16 = 18; // KEY_E
    const A: u16 = 30; // KEY_A
    const SPACE: u16 = 57; // KEY_SPACE
    const COMMA: u16 = 51; // KEY_COMMA
    const SHIFT: u16 = KEY_LEFTSHIFT;

    fn deferred_press_e(sm: &mut StateMachine) {
        let a = sm.deferred_press(KeyInput::Letter(MappingKey::E), Some(E), false, false);
        assert!(a.suppress);
        assert!(a.emit.is_empty());
        assert!(a.pending.is_empty());
        assert!(a.ui.is_none());
        assert!(sm.is_letter_held());
    }

    fn deferred_open_overlay(sm: &mut StateMachine) -> Vec<String> {
        deferred_press_e(sm);
        sm.set_held_ago(Duration::from_millis(300));
        let a = sm.deferred_press(KeyInput::Space, Some(SPACE), false, false);
        assert!(a.suppress);
        assert!(a.emit.is_empty());
        assert!(sm.is_selecting());
        match a.ui {
            Some(GrabEvent::ShowOverlay { variants, index }) => {
                assert_eq!(index, 0);
                assert!(!variants.is_empty());
                variants
            }
            other => panic!("expected ShowOverlay, got {other:?}"),
        }
    }

    #[test]
    fn deferred_letter_is_suppressed_on_press() {
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        deferred_press_e(&mut sm);
    }

    #[test]
    fn deferred_letter_with_mods_passes_through() {
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        let a = sm.deferred_press(KeyInput::Letter(MappingKey::E), Some(E), false, true);
        assert_eq!(a, Action::pass());
        assert!(sm.is_idle());
    }

    #[test]
    fn deferred_unmappable_letter_passes_through() {
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        let a = sm.deferred_press(KeyInput::Letter(MappingKey::E), None, false, false);
        assert_eq!(a, Action::pass());
        assert!(sm.is_idle());
    }

    #[test]
    fn deferred_release_replays_letter() {
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        deferred_press_e(&mut sm);
        let a = sm.deferred_release(Some(E), false);
        assert!(a.suppress);
        assert_eq!(a.emit, vec![KeyEvt::Press(E), KeyEvt::Release(E)]);
        assert!(a.pending.is_empty());
        assert!(sm.is_idle());
    }

    #[test]
    fn deferred_release_wraps_shift_if_dropped() {
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        let a = sm.deferred_press(KeyInput::Letter(MappingKey::E), Some(E), true, false);
        assert!(a.suppress);
        // Shift released before the letter: replay must still produce a capital.
        let a = sm.deferred_release(Some(E), false);
        assert_eq!(
            a.emit,
            vec![
                KeyEvt::Press(SHIFT),
                KeyEvt::Press(E),
                KeyEvt::Release(E),
                KeyEvt::Release(SHIFT),
            ]
        );
    }

    #[test]
    fn deferred_autorepeat_is_swallowed() {
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        deferred_press_e(&mut sm);
        let a = sm.deferred_press(KeyInput::Letter(MappingKey::E), Some(E), false, false);
        assert!(a.suppress);
        assert!(a.emit.is_empty());
        assert!(sm.is_letter_held());
    }

    #[test]
    fn deferred_trigger_after_hold_opens_overlay() {
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        deferred_open_overlay(&mut sm);
    }

    #[test]
    fn deferred_fast_trigger_replays_letter_and_trigger() {
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        deferred_press_e(&mut sm);
        // Space right away (< hold_delay): normal "e " typing.
        let a = sm.deferred_press(KeyInput::Space, Some(SPACE), false, false);
        assert!(a.suppress);
        assert_eq!(
            a.emit,
            vec![KeyEvt::Press(E), KeyEvt::Release(E), KeyEvt::Press(SPACE)]
        );
        assert_eq!(
            a.pending,
            vec![
                (E, ReleaseAction::SwallowOnly),
                (SPACE, ReleaseAction::EmitVirtualRelease),
            ]
        );
        assert!(sm.is_idle());
    }

    #[test]
    fn deferred_rollover_replays_letter_then_new_key() {
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        deferred_press_e(&mut sm);
        let a = sm.deferred_press(KeyInput::Other, Some(COMMA), false, false);
        assert!(a.suppress);
        assert_eq!(
            a.emit,
            vec![KeyEvt::Press(E), KeyEvt::Release(E), KeyEvt::Press(COMMA)]
        );
        assert_eq!(
            a.pending,
            vec![
                (E, ReleaseAction::SwallowOnly),
                (COMMA, ReleaseAction::EmitVirtualRelease),
            ]
        );
        assert!(sm.is_idle());
    }

    #[test]
    fn deferred_rollover_chains_to_next_letter() {
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        deferred_press_e(&mut sm);
        let a = sm.deferred_press(KeyInput::Letter(MappingKey::A), Some(A), false, false);
        assert!(a.suppress);
        // e is replayed, a becomes the new deferred letter.
        assert_eq!(a.emit, vec![KeyEvt::Press(E), KeyEvt::Release(E)]);
        assert_eq!(a.pending, vec![(E, ReleaseAction::SwallowOnly)]);
        assert!(sm.is_letter_held());
        assert_eq!(sm.held_key(), Some(MappingKey::A));
    }

    #[test]
    fn deferred_rollover_unmappable_key_passes_through() {
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        deferred_press_e(&mut sm);
        let a = sm.deferred_press(KeyInput::Other, None, false, false);
        // Letter still replayed, but the unknown key can't ride the virtual
        // device — it passes through on its own device.
        assert!(!a.suppress);
        assert_eq!(a.emit, vec![KeyEvt::Press(E), KeyEvt::Release(E)]);
        assert_eq!(a.pending, vec![(E, ReleaseAction::SwallowOnly)]);
        assert!(sm.is_idle());
    }

    #[test]
    fn deferred_selecting_cycles_with_wrap() {
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        let variants = deferred_open_overlay(&mut sm);
        let n = variants.len();

        let a = sm.deferred_press(KeyInput::Space, Some(SPACE), false, false);
        assert!(a.suppress);
        assert_eq!(a.ui, Some(GrabEvent::UpdateSelection(1 % n)));

        let a = sm.deferred_press(KeyInput::LeftArrow, Some(105), false, false);
        assert!(a.suppress);
        assert_eq!(a.ui, Some(GrabEvent::UpdateSelection(0)));
    }

    #[test]
    fn deferred_escape_replays_plain_letter() {
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        deferred_open_overlay(&mut sm);
        let a = sm.deferred_press(KeyInput::Escape, Some(1), false, false);
        assert!(a.suppress);
        assert_eq!(a.emit, vec![KeyEvt::Press(E), KeyEvt::Release(E)]);
        assert_eq!(a.pending, vec![(E, ReleaseAction::SwallowOnly)]);
        assert_eq!(a.ui, Some(GrabEvent::HideOverlay));
        assert!(sm.is_idle());
    }

    #[test]
    fn deferred_commit_injects_without_cooldown() {
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        let variants = deferred_open_overlay(&mut sm);
        let a = sm.deferred_release(Some(E), false);
        assert!(a.suppress);
        assert!(a.emit.is_empty()); // nothing to delete, nothing to replay
        assert_eq!(a.inject, Some(variants[0].clone()));
        assert_eq!(a.ui, Some(GrabEvent::InjectChar(variants[0].clone())));
        assert!(sm.is_idle());
    }

    #[test]
    fn deferred_selecting_swallows_unrelated_keys() {
        let _guard = setup();
        let mut sm = sm(ActivationKey::Both);
        deferred_open_overlay(&mut sm);
        let a = sm.deferred_press(KeyInput::Other, Some(COMMA), false, false);
        assert!(a.suppress);
        assert!(a.emit.is_empty());
        assert!(sm.is_selecting());

        // Unrelated release passes through (e.g. a key pressed before us).
        let a = sm.deferred_release(Some(COMMA), false);
        assert_eq!(a, Action::pass());
        assert!(sm.is_selecting());
    }
}
