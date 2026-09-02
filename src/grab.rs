use std::cell::RefCell;
use tokio::sync::mpsc::UnboundedSender;

use crate::config::{ActivationKey, Config};
use crate::injection;
use crate::state_machine::{GrabEvent, KeyInput, StateMachine};

#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use core_foundation::base::TCFType;
    use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
    use std::os::raw::c_void;
    use std::ptr;

    type CGEventRef = *mut c_void;
    type CGEventTapProxy = *mut c_void;
    type CFMachPortRef = *mut c_void;
    type CFRunLoopSourceRef = core_foundation::runloop::CFRunLoopSourceRef;
    type CFAllocatorRef = *const c_void;

    const K_CG_EVENT_KEY_DOWN: u32 = 10;
    const K_CG_EVENT_KEY_UP: u32 = 11;
    const K_CG_EVENT_FLAGS_CHANGED: u32 = 12;
    const K_CG_KEYBOARD_EVENT_KEYCODE: u32 = 9;
    const K_CG_EVENT_FLAG_SHIFT: u64 = 0x00020000;

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventTapCreate(
            tap: u32,
            place: u32,
            options: u32,
            events_of_interest: u64,
            callback: extern "C" fn(CGEventTapProxy, u32, CGEventRef, *mut c_void) -> CGEventRef,
            user_info: *mut c_void,
        ) -> CFMachPortRef;
        fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
        fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
        fn CGEventGetFlags(event: CGEventRef) -> u64;
        fn CGEventSourceKeyState(stateID: i32, key: u16) -> bool;
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFMachPortCreateRunLoopSource(
            allocator: CFAllocatorRef,
            port: CFMachPortRef,
            order: i64,
        ) -> CFRunLoopSourceRef;
    }

    use crate::mappings::MappingKey;

    fn keycode_to_input(code: i64) -> KeyInput {
        match code {
            0 => KeyInput::Letter(MappingKey::A),
            1 => KeyInput::Letter(MappingKey::S),
            2 => KeyInput::Letter(MappingKey::D),
            3 => KeyInput::Letter(MappingKey::F),
            4 => KeyInput::Letter(MappingKey::H),
            5 => KeyInput::Letter(MappingKey::G),
            6 => KeyInput::Letter(MappingKey::Z),
            7 => KeyInput::Letter(MappingKey::X),
            8 => KeyInput::Letter(MappingKey::C),
            9 => KeyInput::Letter(MappingKey::V),
            11 => KeyInput::Letter(MappingKey::B),
            12 => KeyInput::Letter(MappingKey::Q),
            13 => KeyInput::Letter(MappingKey::W),
            14 => KeyInput::Letter(MappingKey::E),
            15 => KeyInput::Letter(MappingKey::R),
            16 => KeyInput::Letter(MappingKey::Y),
            17 => KeyInput::Letter(MappingKey::T),
            31 => KeyInput::Letter(MappingKey::O),
            32 => KeyInput::Letter(MappingKey::U),
            34 => KeyInput::Letter(MappingKey::I),
            35 => KeyInput::Letter(MappingKey::P),
            37 => KeyInput::Letter(MappingKey::L),
            38 => KeyInput::Letter(MappingKey::J),
            40 => KeyInput::Letter(MappingKey::K),
            45 => KeyInput::Letter(MappingKey::N),
            46 => KeyInput::Letter(MappingKey::M),
            49 => KeyInput::Space,
            53 => KeyInput::Escape,
            123 => KeyInput::LeftArrow,
            124 => KeyInput::RightArrow,
            _ => KeyInput::Other,
        }
    }

    /// Inverse mapping: MappingKey → macOS keycode (for physical key verification)
    fn mapping_key_to_keycode(mk: MappingKey) -> u16 {
        match mk {
            MappingKey::A => 0,
            MappingKey::S => 1,
            MappingKey::D => 2,
            MappingKey::F => 3,
            MappingKey::H => 4,
            MappingKey::G => 5,
            MappingKey::Z => 6,
            MappingKey::X => 7,
            MappingKey::C => 8,
            MappingKey::V => 9,
            MappingKey::B => 11,
            MappingKey::Q => 12,
            MappingKey::W => 13,
            MappingKey::E => 14,
            MappingKey::R => 15,
            MappingKey::Y => 16,
            MappingKey::T => 17,
            MappingKey::O => 31,
            MappingKey::U => 32,
            MappingKey::I => 34,
            MappingKey::P => 35,
            MappingKey::L => 37,
            MappingKey::J => 38,
            MappingKey::K => 40,
            MappingKey::N => 45,
            MappingKey::M => 46,
        }
    }

    fn is_key_physically_held(keycode: u16) -> bool {
        // kCGEventSourceStateCombinedSessionState = 0
        unsafe { CGEventSourceKeyState(0, keycode) }
    }

    fn is_trigger_input(input: KeyInput) -> bool {
        matches!(input, KeyInput::Space | KeyInput::LeftArrow | KeyInput::RightArrow)
    }

    struct TapContext {
        state: RefCell<StateMachine>,
        shift_held: RefCell<bool>,
        tx: UnboundedSender<GrabEvent>,
    }

    extern "C" fn tap_callback(
        _proxy: CGEventTapProxy,
        event_type: u32,
        event: CGEventRef,
        user_info: *mut c_void,
    ) -> CGEventRef {
        let ctx = unsafe { &*(user_info as *const TapContext) };

        match event_type {
            K_CG_EVENT_FLAGS_CHANGED => {
                let flags = unsafe { CGEventGetFlags(event) };
                let new_shift = (flags & K_CG_EVENT_FLAG_SHIFT) != 0;
                let changed = *ctx.shift_held.borrow() != new_shift;
                *ctx.shift_held.borrow_mut() = new_shift;
                // update_shift is edge-triggered: FLAGS_CHANGED also fires
                // for Ctrl/Alt/Cmd, which must not toggle the picker case.
                if changed {
                    if let Some(ge) = ctx.state.borrow_mut().update_shift(new_shift) {
                        ctx.tx.send(ge).ok();
                    }
                }
                event
            }
            K_CG_EVENT_KEY_DOWN => {
                let keycode_raw =
                    unsafe { CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE) };
                let input = keycode_to_input(keycode_raw);

                // Physical key verification: if in LetterHeld and a trigger arrives,
                // verify the letter is still physically held
                if is_trigger_input(input) {
                    if let Some(held_mk) = ctx.state.borrow().held_key() {
                        let held_keycode = mapping_key_to_keycode(held_mk);
                        if !is_key_physically_held(held_keycode) {
                            eprintln!(
                                "[QuickAccent] Physical key check failed for {:?}, resetting",
                                held_mk
                            );
                            ctx.state.borrow_mut().force_reset();
                            return event; // pass through
                        }
                    }
                }

                let (suppress, grab_event) = ctx
                    .state
                    .borrow_mut()
                    .handle_key_press(input, *ctx.shift_held.borrow());
                if let Some(ref ge) = grab_event {
                    eprintln!(
                        "[QuickAccent] KeyDown {} -> {:?}, suppress: {}",
                        keycode_raw, ge, suppress
                    );
                    ctx.tx.send(ge.clone()).ok();
                }
                if suppress { ptr::null_mut() } else { event }
            }
            K_CG_EVENT_KEY_UP => {
                let keycode_raw =
                    unsafe { CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE) };
                let input = keycode_to_input(keycode_raw);
                let (suppress, grab_event) =
                    ctx.state.borrow_mut().handle_key_release(input);
                if let Some(ref ge) = grab_event {
                    eprintln!(
                        "[QuickAccent] KeyUp {} -> {:?}, suppress: {}",
                        keycode_raw, ge, suppress
                    );
                    match ge {
                        GrabEvent::InjectChar(ch) => injection::inject_char(ch.clone()),
                        GrabEvent::FalseStart => injection::inject_space(),
                        _ => {}
                    }
                    ctx.tx.send(ge.clone()).ok();
                }
                if suppress { ptr::null_mut() } else { event }
            }
            _ => event,
        }
    }

    pub fn run_grab(tx: UnboundedSender<GrabEvent>, input_time_ms: u64, hold_delay_ms: u64, activation_key: ActivationKey) {
        let ctx = Box::new(TapContext {
            state: RefCell::new(StateMachine::new(input_time_ms, hold_delay_ms, activation_key)),
            shift_held: RefCell::new(false),
            tx,
        });
        let ctx_ptr = Box::into_raw(ctx) as *mut c_void;

        let mask = (1u64 << K_CG_EVENT_KEY_DOWN)
            | (1u64 << K_CG_EVENT_KEY_UP)
            | (1u64 << K_CG_EVENT_FLAGS_CHANGED);

        let tap_port = unsafe {
            CGEventTapCreate(
                0, // kCGHIDEventTap
                0, // kCGHeadInsertEventTap
                0, // kCGEventTapOptionDefault
                mask,
                tap_callback,
                ctx_ptr,
            )
        };

        if tap_port.is_null() {
            eprintln!("[QuickAccent] ERROR: Failed to create CGEventTap.");
            eprintln!("[QuickAccent] Grant Accessibility permissions to this terminal.");
            return;
        }

        unsafe {
            let source_ref = CFMachPortCreateRunLoopSource(ptr::null(), tap_port, 0);
            assert!(!source_ref.is_null(), "Failed to create run loop source");
            let source =
                core_foundation::runloop::CFRunLoopSource::wrap_under_create_rule(source_ref);
            let run_loop = CFRunLoop::get_current();
            run_loop.add_source(&source, kCFRunLoopCommonModes);
            CGEventTapEnable(tap_port, true);
        }

        eprintln!("[QuickAccent] CGEventTap active. Listening for keys...");
        CFRunLoop::run_current();
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::*;
    use crate::mappings::MappingKey;
    use crate::state_machine::{Action, KeyEvt, ReleaseAction};
    use crate::virtual_kb;
    use crate::xkb_map;
    use rdev::{grab_with_is_repeat, Event, EventType, Key};
    use std::collections::HashMap;

    /// Physically held modifiers, tracked from the raw event stream.
    #[derive(Default)]
    struct Mods {
        shift_l: bool,
        shift_r: bool,
        ctrl_l: bool,
        ctrl_r: bool,
        alt: bool,
        altgr: bool,
        meta_l: bool,
        meta_r: bool,
    }

    impl Mods {
        /// Update from a key event; returns true if it was a modifier.
        fn update(&mut self, key: Key, pressed: bool) -> bool {
            match key {
                Key::ShiftLeft => self.shift_l = pressed,
                Key::ShiftRight => self.shift_r = pressed,
                Key::ControlLeft => self.ctrl_l = pressed,
                Key::ControlRight => self.ctrl_r = pressed,
                Key::Alt => self.alt = pressed,
                Key::AltGr => self.altgr = pressed,
                Key::MetaLeft => self.meta_l = pressed,
                Key::MetaRight => self.meta_r = pressed,
                _ => return false,
            }
            true
        }

        fn shift(&self) -> bool {
            self.shift_l || self.shift_r
        }

        /// Any non-shift modifier held (shortcut chords bypass accents).
        fn nonshift(&self) -> bool {
            self.ctrl_l || self.ctrl_r || self.alt || self.altgr || self.meta_l || self.meta_r
        }

        /// evdev codes of the held Shift keys (for neutralization).
        fn held_shift_codes(&self) -> Vec<u16> {
            use crate::state_machine::{KEY_LEFTSHIFT, KEY_RIGHTSHIFT};
            let mut v = Vec::new();
            if self.shift_l {
                v.push(KEY_LEFTSHIFT);
            }
            if self.shift_r {
                v.push(KEY_RIGHTSHIFT);
            }
            v
        }
    }

    fn physical_letter(key: Key) -> Option<MappingKey> {
        match key {
            Key::KeyA => Some(MappingKey::A),
            Key::KeyB => Some(MappingKey::B),
            Key::KeyC => Some(MappingKey::C),
            Key::KeyD => Some(MappingKey::D),
            Key::KeyE => Some(MappingKey::E),
            Key::KeyF => Some(MappingKey::F),
            Key::KeyG => Some(MappingKey::G),
            Key::KeyH => Some(MappingKey::H),
            Key::KeyI => Some(MappingKey::I),
            Key::KeyJ => Some(MappingKey::J),
            Key::KeyK => Some(MappingKey::K),
            Key::KeyL => Some(MappingKey::L),
            Key::KeyM => Some(MappingKey::M),
            Key::KeyN => Some(MappingKey::N),
            Key::KeyO => Some(MappingKey::O),
            Key::KeyP => Some(MappingKey::P),
            Key::KeyQ => Some(MappingKey::Q),
            Key::KeyR => Some(MappingKey::R),
            Key::KeyS => Some(MappingKey::S),
            Key::KeyT => Some(MappingKey::T),
            Key::KeyU => Some(MappingKey::U),
            Key::KeyV => Some(MappingKey::V),
            Key::KeyW => Some(MappingKey::W),
            Key::KeyX => Some(MappingKey::X),
            Key::KeyY => Some(MappingKey::Y),
            Key::KeyZ => Some(MappingKey::Z),
            _ => None,
        }
    }

    fn event_to_input(event: &Event) -> KeyInput {
        let (EventType::KeyPress(key) | EventType::KeyRelease(key)) = event.event_type else {
            return KeyInput::Other;
        };
        match key {
            Key::Space => KeyInput::Space,
            Key::Escape => KeyInput::Escape,
            Key::LeftArrow => KeyInput::LeftArrow,
            Key::RightArrow => KeyInput::RightArrow,
            other => {
                if let Some(mk) = event.name.as_deref().and_then(xkb_map::letter_from_name) {
                    return KeyInput::Letter(mk);
                }
                match physical_letter(other) {
                    Some(p) => KeyInput::Letter(xkb_map::logical_letter(p)),
                    None => KeyInput::Other,
                }
            }
        }
    }

    /// Keep the virtual keyboard's modifier state in lockstep with the
    /// physical one. Hyprland tracks modifiers per keyboard device and reports
    /// the *emitting* device's state to clients, so a letter replayed through
    /// our device while Shift is held on the physical one comes out lowercase
    /// (issue #9: "uPeRcaseS" — vowels are the replayed keys). Mirroring the
    /// matched press/release pairs, in order, is a no-op where state is merged
    /// (mutter) and correct where it is per device. Caps Lock is deliberately
    /// excluded: its xkb action toggles on press, so a mirrored second press
    /// would undo it on merged-state compositors.
    fn mirror_modifier(code: Option<u16>, pressed: bool, is_repeat: bool) {
        if is_repeat {
            return;
        }
        if let Some(c) = code {
            let ev = if pressed { KeyEvt::Press(c) } else { KeyEvt::Release(c) };
            if let Err(e) = virtual_kb::emit(&[ev]) {
                eprintln!("[QuickAccent] modifier mirror failed: {e}");
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// `Mods::update` defines exactly the set that gets mirrored.
        #[test]
        fn mirrored_modifier_set_excludes_capslock_and_letters() {
            let mut m = Mods::default();
            for k in [
                Key::ShiftLeft, Key::ShiftRight, Key::ControlLeft, Key::ControlRight,
                Key::Alt, Key::AltGr, Key::MetaLeft, Key::MetaRight,
            ] {
                assert!(m.update(k, true), "{k:?} must be tracked/mirrored");
            }
            assert!(!m.update(Key::CapsLock, true), "CapsLock must never be mirrored");
            assert!(!m.update(Key::KeyE, true));
            assert!(!m.update(Key::Space, true));
        }
    }

    /// Type the committed accent character, most direct mechanism first:
    /// 1. uinput key combo — char exists in the active keyboard layout;
    /// 2. portal keysym — direct Unicode injection by the compositor
    ///    (PowerAccent's SendInput(KEYEVENTF_UNICODE) equivalent);
    /// 3. clipboard paste — emergency fallback only.
    fn inject_commit(ch: &str, mods: &Mods) {
        let held_shifts = mods.held_shift_codes();
        let mut chars = ch.chars();
        let single = match (chars.next(), chars.next()) {
            (Some(c), None) => Some(c),
            _ => None,
        };
        if let Some(combo) = single.and_then(xkb_map::combo_for_char) {
            match virtual_kb::emit_combo(
                combo.code,
                combo.shift,
                combo.altgr,
                &held_shifts,
                mods.altgr,
            ) {
                Ok(()) => return,
                Err(e) => eprintln!("[QuickAccent] direct injection failed: {e}"),
            }
        }
        match crate::portal_keysym::inject_text_sync(ch) {
            Ok(()) => return,
            Err(crate::portal_keysym::InjectError::Timeout) => {
                // The character may still arrive — injecting through another
                // path too would double it. Do nothing.
                eprintln!("[QuickAccent] portal injection timed out; not falling back");
                return;
            }
            Err(crate::portal_keysym::InjectError::Unavailable(e)) => {
                eprintln!("[QuickAccent] portal injection unavailable: {e}");
            }
        }
        injection::inject_char_fallback(ch.to_string(), held_shifts);
    }

    pub fn run_grab(
        tx: UnboundedSender<GrabEvent>,
        input_time_ms: u64,
        hold_delay_ms: u64,
        activation_key: ActivationKey,
    ) {
        // The grab swallows keystrokes and replays them through the virtual
        // keyboard — never grab without it, or we'd eat the user's typing.
        if let Err(e) = virtual_kb::init() {
            eprintln!(
                "[QuickAccent] virtual keyboard unavailable: {e}\n\
                 QuickAccent needs /dev/uinput and /dev/input access.\n\
                 Run ./dist/linux/install.sh, ensure you are in the 'input' group\n\
                 (log out/in or reboot), and that the uinput module is loaded\n\
                 (sudo modprobe uinput). Keyboard grabbing is disabled."
            );
            return;
        }
        // Our own device must not be grabbed: injected events go straight to
        // the compositor instead of looping back through this callback.
        rdev::grab_skip_device_named(virtual_kb::DEVICE_NAME);

        let _ = xkb_map::logical_letter(MappingKey::A); // warm layout map

        // Supervised grab: `grab()` returns when the event loop ends — which
        // includes suspend/resume, where the kernel can hand back an error we
        // can't paper over inside the loop. If the thread just returned here,
        // the process would stay alive but deaf (issue: "running but not
        // listening after suspend"). So re-establish the grab instead —
        // rebuilding fresh state (Idle is the right reset) and re-running
        // setup_devices, which re-opens and re-EVIOCGRABs every keyboard,
        // recovering from re-enumeration too. A grab that keeps failing
        // immediately (permission genuinely gone) backs off and finally exits
        // non-zero so systemd restarts us rather than spinning.
        let mut fast_failures = 0u32;
        loop {
            let started = std::time::Instant::now();
            run_grab_once(input_time_ms, hold_delay_ms, activation_key, &tx);
            let ran = started.elapsed();
            if ran >= std::time::Duration::from_secs(5) {
                // It was healthy for a while — a resume/hotplug blip. Retry now.
                fast_failures = 0;
                eprintln!("[QuickAccent] input grab dropped after {ran:?}; re-establishing");
                continue;
            }
            fast_failures += 1;
            if fast_failures >= 5 {
                eprintln!(
                    "[QuickAccent] input grab keeps failing immediately; exiting so the \
                     service manager can restart it (check the 'input' group and /dev/uinput)"
                );
                std::process::exit(1);
            }
            let backoff = std::time::Duration::from_millis(200 * u64::from(fast_failures));
            eprintln!("[QuickAccent] input grab failed; retrying in {backoff:?}");
            std::thread::sleep(backoff);
        }
    }

    /// One run of the evdev grab. Returns when the underlying event loop ends
    /// (error, or all devices gone); the caller decides whether to retry.
    fn run_grab_once(
        input_time_ms: u64,
        hold_delay_ms: u64,
        activation_key: ActivationKey,
        tx: &UnboundedSender<GrabEvent>,
    ) {
        let tx = tx.clone();
        let state = RefCell::new(StateMachine::new(input_time_ms, hold_delay_ms, activation_key));
        let mods = RefCell::new(Mods::default());
        let pending_release: RefCell<HashMap<u16, ReleaseAction>> = RefCell::new(HashMap::new());

        let callback = move |event: Event, is_repeat: bool| -> Option<Event> {
            let (key, pressed) = match event.event_type {
                EventType::KeyPress(k) => (k, true),
                EventType::KeyRelease(k) => (k, false),
                _ => return Some(event),
            };
            let code = xkb_map::evdev_code_of(key);

            // 1. Autorepeat of a key we already replayed virtually (still
            // physically held): swallow. Apps synthesize their own repeats
            // from key state, kernel repeat events are ignored downstream.
            if is_repeat {
                if let Some(c) = code {
                    if pending_release.borrow().contains_key(&c) {
                        return None;
                    }
                }
            }

            // Track physical modifier state before anything can intercept —
            // a suppressed physical Ctrl-up must still clear the tracker.
            let shift_before = mods.borrow().shift();
            let is_modifier = mods.borrow_mut().update(key, pressed);

            // 2. Physical releases of keys we already replayed virtually.
            if !pressed {
                if let Some(c) = code {
                    if let Some(ra) = pending_release.borrow_mut().remove(&c) {
                        if ra == ReleaseAction::EmitVirtualRelease {
                            if let Err(e) = virtual_kb::emit(&[KeyEvt::Release(c)]) {
                                eprintln!("[QuickAccent] replay failed: {e}");
                            }
                        }
                        return None;
                    }
                }
            }

            // 3. Shift passes through untouched (it drives live case
            // switching in the overlay); other modifiers fall through to the
            // state machine so a chord during LetterHeld replays the letter.
            if matches!(key, Key::ShiftLeft | Key::ShiftRight) {
                mirror_modifier(code, pressed, is_repeat);
                let shift = mods.borrow().shift();
                // update_shift is edge-triggered (a press toggles the picker
                // case): kernel autorepeat of a held Shift, or the second
                // Shift key while one is down, must not reach it.
                if shift != shift_before {
                    if let Some(ge) = state.borrow_mut().update_shift(shift) {
                        let _ = tx.send(ge);
                    }
                }
                return Some(event);
            }

            // 4. State machine (deferred mode).
            let (shift, nonshift_mods, input) = {
                let m = mods.borrow();
                // A modifier's own keydown isn't "typed with a modifier held".
                let nonshift = if is_modifier { false } else { m.nonshift() };
                (m.shift(), nonshift, event_to_input(&event))
            };
            let action: Action = if pressed {
                state
                    .borrow_mut()
                    .deferred_press(input, code, shift, nonshift_mods)
            } else {
                state.borrow_mut().deferred_release(code, shift)
            };

            // 5. Execute: replay first (ordering!), then bookkeeping, then
            // commit injection, then UI.
            if !action.emit.is_empty() {
                if let Err(e) = virtual_kb::emit(&action.emit) {
                    eprintln!("[QuickAccent] replay failed: {e}");
                }
            }
            for (c, ra) in &action.pending {
                pending_release.borrow_mut().insert(*c, *ra);
            }
            if let Some(ch) = &action.inject {
                inject_commit(ch, &mods.borrow());
            }
            if is_modifier && !action.suppress {
                // Ctrl/Alt/Meta/AltGr passing through: keep the virtual
                // keyboard's state in step (suppressed ones were already
                // emitted virtually by the rollover path).
                mirror_modifier(code, pressed, is_repeat);
            }
            if let Some(ge) = action.ui {
                if matches!(ge, GrabEvent::ShowOverlay { .. }) {
                    // Anchor the overlay on the window being typed in, so it
                    // opens on the right monitor (needs the shell extension;
                    // None keeps the primary-centered fallback).
                    crate::app::set_overlay_anchor(
                        crate::shell_ext::focused_window_rect()
                            .map(|r| (r.x, r.y, r.width, r.height)),
                    );
                }
                let _ = tx.send(ge);
            }
            if action.suppress {
                None
            } else {
                Some(event)
            }
        };

        if let Err(e) = grab_with_is_repeat(callback) {
            eprintln!("[QuickAccent] grab error: {e:?}");
        }
    }
}

pub fn run_grab_thread(tx: UnboundedSender<GrabEvent>, config: &Config) {
    let input_time_ms = config.input_time_ms;
    let hold_delay_ms = config.hold_delay_ms;
    let activation_key = config.activation_key_parsed();
    std::thread::spawn(move || {
        #[cfg(target_os = "macos")]
        eprintln!("[QuickAccent] Starting grab (grant Accessibility if needed)...");
        #[cfg(target_os = "linux")]
        eprintln!("[QuickAccent] Starting grab (user must be in group 'input')...");
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        eprintln!("[QuickAccent] Starting grab...");
        platform::run_grab(tx, input_time_ms, hold_delay_ms, activation_key);
    });
}
