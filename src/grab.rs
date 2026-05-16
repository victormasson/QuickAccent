use std::cell::RefCell;
use tokio::sync::mpsc::UnboundedSender;

use crate::injection;
use crate::state_machine::{AccentState, GrabEvent, KeyInput};

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
    }

    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFMachPortCreateRunLoopSource(
            allocator: CFAllocatorRef,
            port: CFMachPortRef,
            order: i64,
        ) -> CFRunLoopSourceRef;
    }

    fn keycode_to_input(code: i64) -> KeyInput {
        use crate::mappings::MappingKey;
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
            _ => KeyInput::Other,
        }
    }

    struct TapContext {
        state: RefCell<AccentState>,
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
                *ctx.shift_held.borrow_mut() = (flags & K_CG_EVENT_FLAG_SHIFT) != 0;
                event
            }
            K_CG_EVENT_KEY_DOWN => {
                let keycode_raw =
                    unsafe { CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE) };
                let input = keycode_to_input(keycode_raw);
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
                    if let GrabEvent::InjectChar(ch) = ge {
                        injection::inject_char(*ch);
                    }
                    ctx.tx.send(ge.clone()).ok();
                }
                if suppress { ptr::null_mut() } else { event }
            }
            _ => event,
        }
    }

    pub fn run_grab(tx: UnboundedSender<GrabEvent>) {
        let ctx = Box::new(TapContext {
            state: RefCell::new(AccentState::new()),
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
    use rdev::{grab, Event, EventType, Key};

    fn rdev_key_to_input(key: Key) -> KeyInput {
        use crate::mappings::MappingKey;
        match key {
            Key::KeyA => KeyInput::Letter(MappingKey::A),
            Key::KeyB => KeyInput::Letter(MappingKey::B),
            Key::KeyC => KeyInput::Letter(MappingKey::C),
            Key::KeyD => KeyInput::Letter(MappingKey::D),
            Key::KeyE => KeyInput::Letter(MappingKey::E),
            Key::KeyF => KeyInput::Letter(MappingKey::F),
            Key::KeyG => KeyInput::Letter(MappingKey::G),
            Key::KeyH => KeyInput::Letter(MappingKey::H),
            Key::KeyI => KeyInput::Letter(MappingKey::I),
            Key::KeyJ => KeyInput::Letter(MappingKey::J),
            Key::KeyK => KeyInput::Letter(MappingKey::K),
            Key::KeyL => KeyInput::Letter(MappingKey::L),
            Key::KeyM => KeyInput::Letter(MappingKey::M),
            Key::KeyN => KeyInput::Letter(MappingKey::N),
            Key::KeyO => KeyInput::Letter(MappingKey::O),
            Key::KeyP => KeyInput::Letter(MappingKey::P),
            Key::KeyQ => KeyInput::Letter(MappingKey::Q),
            Key::KeyR => KeyInput::Letter(MappingKey::R),
            Key::KeyS => KeyInput::Letter(MappingKey::S),
            Key::KeyT => KeyInput::Letter(MappingKey::T),
            Key::KeyU => KeyInput::Letter(MappingKey::U),
            Key::KeyV => KeyInput::Letter(MappingKey::V),
            Key::KeyW => KeyInput::Letter(MappingKey::W),
            Key::KeyX => KeyInput::Letter(MappingKey::X),
            Key::KeyY => KeyInput::Letter(MappingKey::Y),
            Key::KeyZ => KeyInput::Letter(MappingKey::Z),
            Key::Space => KeyInput::Space,
            Key::Escape => KeyInput::Escape,
            _ => KeyInput::Other,
        }
    }

    pub fn run_grab(tx: UnboundedSender<GrabEvent>) {
        let state = RefCell::new(AccentState::new());
        let shift_held = RefCell::new(false);

        let callback = move |event: Event| -> Option<Event> {
            match event.event_type {
                EventType::KeyPress(Key::ShiftLeft | Key::ShiftRight) => {
                    *shift_held.borrow_mut() = true;
                    Some(event)
                }
                EventType::KeyRelease(Key::ShiftLeft | Key::ShiftRight) => {
                    *shift_held.borrow_mut() = false;
                    Some(event)
                }
                EventType::KeyPress(key) => {
                    let input = rdev_key_to_input(key);
                    let (suppress, grab_event) =
                        state
                            .borrow_mut()
                            .handle_key_press(input, *shift_held.borrow());
                    if let Some(ref ge) = grab_event {
                        if let GrabEvent::InjectChar(ch) = ge {
                            injection::inject_char(*ch);
                        }
                        tx.send(ge.clone()).ok();
                    }
                    if suppress { None } else { Some(event) }
                }
                EventType::KeyRelease(key) => {
                    let input = rdev_key_to_input(key);
                    let (suppress, grab_event) =
                        state.borrow_mut().handle_key_release(input);
                    if let Some(ref ge) = grab_event {
                        if let GrabEvent::InjectChar(ch) = ge {
                            injection::inject_char(*ch);
                        }
                        tx.send(ge.clone()).ok();
                    }
                    if suppress { None } else { Some(event) }
                }
                _ => Some(event),
            }
        };

        match grab(callback) {
            Ok(()) => eprintln!("[QuickAccent] Grab ended normally."),
            Err(e) => eprintln!("[QuickAccent] ERROR: Grab failed: {:?}", e),
        }
    }
}

pub fn run_grab_thread(tx: UnboundedSender<GrabEvent>) {
    std::thread::spawn(move || {
        eprintln!("[QuickAccent] Starting keyboard grab...");
        eprintln!("[QuickAccent] Make sure Accessibility permissions are granted.");
        platform::run_grab(tx);
    });
}
