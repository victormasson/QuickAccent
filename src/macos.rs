use cocoa::appkit::{NSApp, NSApplication, NSImage, NSMenu, NSMenuItem, NSStatusBar};
use cocoa::base::{id, nil, selector};
use cocoa::foundation::NSString;
use objc::runtime::Class;
use objc::{msg_send, sel, sel_impl};

pub fn setup_status_item() {
    unsafe {
        let app = NSApp();
        app.setActivationPolicy_(
            cocoa::appkit::NSApplicationActivationPolicy::NSApplicationActivationPolicyAccessory,
        );

        let status_bar = NSStatusBar::systemStatusBar(nil);
        let status_item: id = msg_send![status_bar, statusItemWithLength: -1.0];

        // Menu bar icon. NSImage cannot decode SVG data — only asset-catalog
        // SVGs are supported — so this must be a raster image; an SVG here
        // yields a nil image and an invisible status item. The PNG is a
        // template (black + alpha), which macOS recolours to match light and
        // dark menu bars automatically.
        let icon_data = include_bytes!("../assets/menubar-template.png");
        let ns_data: id = msg_send![Class::get("NSData").unwrap(), dataWithBytes: icon_data.as_ptr() length: icon_data.len()];
        let ns_image: id = msg_send![NSImage::alloc(nil), initWithData: ns_data];

        let button: id = msg_send![status_item, button];
        if ns_image != nil {
            // 36px artwork drawn at 18pt → crisp on Retina.
            let size = cocoa::foundation::NSSize::new(18.0, 18.0);
            let _: () = msg_send![ns_image, setSize: size];
            let _: () = msg_send![ns_image, setTemplate: true];
            let _: () = msg_send![button, setImage: ns_image];
        } else {
            // Never leave an invisible status item behind.
            let fallback = NSString::alloc(nil).init_str("Q\u{304}");
            let _: () = msg_send![button, setTitle: fallback];
        }

        // Menu with Quit
        let menu = NSMenu::new(nil);
        let quit_title = NSString::alloc(nil).init_str("Quit QuickAccent");
        let q = NSString::alloc(nil).init_str("q");
        let quit_item: id = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
            quit_title,
            selector("terminate:"),
            q,
        );
        menu.addItem_(quit_item);

        let _: () = msg_send![status_item, setMenu: menu];
    }
}
