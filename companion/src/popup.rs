use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use gtk::prelude::*;
use gtk::{gdk, glib};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::store::Listeners;
use crate::widgets::anim::{self, Tween};
use crate::widgets::motion::{Motion, Origin};

/// The bar sits on the left, so its menus open beside it; anchoring the left
/// edge only (no margin) puts them right after the bar's exclusive zone.
#[derive(Clone, Copy)]
pub enum Place {
    /// Beside the bottom of the bar, where the control group lives.
    BarBottom,
    /// Beside the middle of the bar, where the clock lives.
    BarMiddle,
    /// Docked to the bottom edge of the screen.
    BottomEdge,
    /// The middle of the screen, for the clipboard and power menus.
    Center,
    /// Near the top, so the launcher grows downwards as results come in.
    Top,
}

const GAP: i32 = 8;
const LEAVE_DELAY: Duration = Duration::from_millis(350);

pub struct Popup {
    pub window: gtk::Window,
    place: Place,
    motion: Motion,
    tween: Rc<Tween>,
    open: Rc<Cell<bool>>,
    sticky: Cell<bool>,
    leave: RefCell<Option<glib::SourceId>>,
    listeners: Listeners<bool>,
}

impl Popup {
    pub fn new(
        app: &gtk::Application,
        name: &str,
        place: Place,
        interactive: bool,
        child: &impl IsA<gtk::Widget>,
    ) -> Rc<Self> {
        // Not resizable: a window then always takes its content's natural
        // size, so it shrinks back after something big has passed through.
        let window = gtk::Window::builder()
            .application(app)
            .decorated(false)
            .resizable(false)
            .build();
        window.add_css_class("octopus-popup");
        window.init_layer_shell();
        window.set_namespace(Some(&format!("octopus-{name}")));
        // The top layer, like the bar: a fullscreen window covers it.
        window.set_layer(Layer::Top);
        window.set_keyboard_mode(if interactive {
            KeyboardMode::Exclusive
        } else {
            KeyboardMode::None
        });

        let origin = match place {
            Place::BarBottom | Place::BarMiddle => {
                window.set_anchor(Edge::Left, true);
                window.set_margin(Edge::Left, GAP);
                if matches!(place, Place::BarBottom) {
                    window.set_anchor(Edge::Bottom, true);
                    window.set_margin(Edge::Bottom, GAP);
                    Origin::LeftBottom
                } else {
                    Origin::Left
                }
            }
            Place::Center => Origin::Fade,
            Place::Top => {
                window.set_anchor(Edge::Top, true);
                window.set_margin(Edge::Top, 148);
                Origin::Fade
            }
            Place::BottomEdge => {
                window.set_anchor(Edge::Bottom, true);
                window.set_margin(Edge::Bottom, -6);
                Origin::BottomEdge
            }
        };

        let motion = Motion::new(child, origin);
        window.set_child(Some(&motion));

        let popup = Rc::new(Self {
            window,
            place,
            motion,
            tween: Tween::new(0.0),
            open: Rc::new(Cell::new(false)),
            sticky: Cell::new(false),
            leave: RefCell::new(None),
            listeners: RefCell::new(Vec::new()),
        });

        if interactive {
            let keys = gtk::EventControllerKey::new();
            keys.set_propagation_phase(gtk::PropagationPhase::Capture);
            let weak = Rc::downgrade(&popup);
            keys.connect_key_pressed(move |_, key, _, _| {
                if key == gdk::Key::Escape {
                    if let Some(popup) = weak.upgrade() {
                        popup.hide();
                    }
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            });
            popup.window.add_controller(keys);
            popup.close_on_leave();
        }

        popup
    }

    /// Leaving the panel with the pointer closes it, once the pointer has
    /// been inside, unless something is being typed.
    fn close_on_leave(self: &Rc<Self>) {
        let pointer = gtk::EventControllerMotion::new();
        let weak = Rc::downgrade(self);
        pointer.connect_enter(move |_, _, _| {
            if let Some(popup) = weak.upgrade() {
                if let Some(id) = popup.leave.borrow_mut().take() {
                    id.remove();
                }
            }
        });
        let weak = Rc::downgrade(self);
        pointer.connect_leave(move |_| {
            let Some(popup) = weak.upgrade() else { return };
            if !popup.is_open() || popup.sticky.get() || popup.typing() {
                return;
            }
            let later = Rc::downgrade(&popup);
            let id = glib::timeout_add_local_once(LEAVE_DELAY, move || {
                if let Some(popup) = later.upgrade() {
                    popup.leave.borrow_mut().take();
                    popup.hide();
                }
            });
            let old = popup.leave.borrow_mut().replace(id);
            if let Some(old) = old {
                old.remove();
            }
        });
        self.window.add_controller(pointer);
    }

    fn typing(&self) -> bool {
        gtk::prelude::GtkWindowExt::focus(&self.window)
            .and_then(|w| w.downcast::<gtk::Text>().ok())
            .is_some_and(|text| !text.text().is_empty())
    }

    /// Moves a bar panel to the side the bar is on.
    pub fn set_side(&self, right: bool) {
        if !matches!(self.place, Place::BarBottom | Place::BarMiddle) {
            return;
        }
        let (near, far) = if right {
            (Edge::Right, Edge::Left)
        } else {
            (Edge::Left, Edge::Right)
        };
        self.window.set_anchor(far, false);
        self.window.set_margin(far, 0);
        self.window.set_anchor(near, true);
        self.window.set_margin(near, GAP);
        self.motion.set_origin(match (right, self.place) {
            (true, _) => Origin::Right,
            (false, Place::BarBottom) => Origin::LeftBottom,
            _ => Origin::Left,
        });
    }

    /// A sticky panel stays when the pointer leaves or clicks outside;
    /// only its own keys (Enter, Escape) or a command close it.
    pub fn set_sticky(&self, on: bool) {
        self.sticky.set(on);
    }

    pub fn is_sticky(&self) -> bool {
        self.sticky.get()
    }

    pub fn is_open(&self) -> bool {
        self.open.get()
    }

    pub fn show(&self) {
        if self.open.replace(true) {
            return;
        }
        self.window.set_visible(true);
        let motion = self.motion.clone();
        self.tween.animate(
            &self.motion,
            1.0,
            300,
            anim::EMPHASIZED_DECELERATE,
            move |p| motion.set_progress(p),
            || {},
        );
        self.emit(true);
    }

    /// Maps the surface once, fully transparent, so the first real opening
    /// does not pay for creating the renderer and laying everything out.
    pub fn warm(&self) {
        if self.open.get() {
            return;
        }
        let mode = self.window.keyboard_mode();
        self.window.set_keyboard_mode(KeyboardMode::None);
        self.motion.set_progress(0.0);
        self.window.set_visible(true);
        let window = self.window.clone();
        let open = self.open.clone();
        glib::timeout_add_local_once(Duration::from_millis(400), move || {
            if !open.get() {
                window.set_visible(false);
            }
            window.set_keyboard_mode(mode);
        });
    }

    pub fn hide(&self) {
        if !self.open.replace(false) {
            return;
        }
        if let Some(id) = self.leave.borrow_mut().take() {
            id.remove();
        }
        let motion = self.motion.clone();
        let window = self.window.clone();
        self.tween.animate(
            &self.motion,
            0.0,
            170,
            anim::EMPHASIZED_ACCELERATE,
            move |p| motion.set_progress(p),
            move || window.set_visible(false),
        );
        self.emit(false);
    }

    pub fn on_visibility(&self, listener: impl Fn(bool) + 'static) {
        self.listeners.borrow_mut().push(Box::new(listener));
    }

    fn emit(&self, open: bool) {
        for listener in self.listeners.borrow().iter() {
            listener(open);
        }
    }
}

/// An invisible full-screen layer under the menus that closes them on a
/// click outside. It keeps the bar's exclusive zone, so the bar stays
/// clickable and can switch straight to another menu.
pub fn backdrop(app: &gtk::Application, on_click: impl Fn() + 'static) -> gtk::Window {
    let window = gtk::Window::builder()
        .application(app)
        .decorated(false)
        .build();
    window.add_css_class("octopus-backdrop");
    window.init_layer_shell();
    window.set_namespace(Some("octopus-backdrop"));
    window.set_layer(Layer::Top);
    window.set_keyboard_mode(KeyboardMode::None);
    for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
        window.set_anchor(edge, true);
    }
    window.set_child(Some(&gtk::Box::new(gtk::Orientation::Vertical, 0)));
    let click = gtk::GestureClick::new();
    click.set_button(0);
    click.connect_pressed(move |_, _, _, _| on_click());
    window.add_controller(click);
    window
}

/// Layer windows grow with their content but never shrink back on their own:
/// asking for the smallest size lets them take the content's size again.
pub fn fit(widget: &impl IsA<gtk::Widget>) {
    let Some(window) = widget.root().and_downcast::<gtk::Window>() else {
        return;
    };
    let Some(child) = window.child() else { return };
    let (_, width, _, _) = child.measure(gtk::Orientation::Horizontal, -1);
    let (_, height, _, _) = child.measure(gtk::Orientation::Vertical, width);
    window.set_default_size(width, height);
}

/// Follows a stack's size on every frame while it changes page.
pub fn follow(stack: &gtk::Stack) {
    stack.connect_transition_running_notify(|stack| {
        fit(stack);
        if stack.is_transition_running() {
            stack.add_tick_callback(|stack, _| {
                fit(stack);
                if stack.is_transition_running() {
                    glib::ControlFlow::Continue
                } else {
                    glib::ControlFlow::Break
                }
            });
        }
    });
}
