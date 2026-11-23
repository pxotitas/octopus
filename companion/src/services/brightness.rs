use std::rc::Rc;

use gtk::glib;

use crate::exec;
use crate::store::Store;

/// A brightnessctl device: the screen backlight or the keyboard LEDs.
pub struct Light {
    device: std::cell::RefCell<String>,
    /// None when the machine has no such device.
    pub percent: Rc<Store<Option<u32>>>,
}

impl Light {
    pub fn new(configured: &str, keyboard: bool) -> Rc<Self> {
        let light = Rc::new(Self {
            device: std::cell::RefCell::new(configured.to_owned()),
            percent: Store::new(None),
        });
        let this = light.clone();
        glib::spawn_future_local(async move {
            if this.device.borrow().is_empty() {
                if let Some(found) = find(keyboard).await {
                    *this.device.borrow_mut() = found;
                }
            }
            this.refresh();
        });
        light
    }

    pub fn refresh(self: &Rc<Self>) {
        let device = self.device.borrow().clone();
        if device.is_empty() {
            return;
        }
        let store = self.percent.clone();
        glib::spawn_future_local(async move {
            let line = exec::output("brightnessctl", &["-m", "-d", &device, "info"]).await;
            store.set(line.and_then(|l| percent(&l)));
        });
    }

    pub fn set(&self, value: u32) {
        let device = self.device.borrow().clone();
        if device.is_empty() {
            return;
        }
        let value = value.clamp(1, 100);
        exec::spawn(
            "brightnessctl",
            &["-q", "-d", &device, "set", &format!("{value}%")],
        );
        self.percent.set(Some(value));
    }

    pub fn step(&self, delta: i32) {
        let current = self.percent.get().unwrap_or(50) as i32;
        self.set((current + delta).clamp(1, 100) as u32);
    }
}

fn percent(line: &str) -> Option<u32> {
    line.trim()
        .split(',')
        .nth(3)?
        .trim_end_matches('%')
        .parse()
        .ok()
}

async fn find(keyboard: bool) -> Option<String> {
    let list = exec::output("brightnessctl", &["-l", "-m"]).await?;
    list.lines().find_map(|line| {
        let mut fields = line.split(',');
        let name = fields.next()?;
        let class = fields.next()?;
        let wanted = if keyboard {
            class == "leds" && name.contains("kbd_backlight")
        } else {
            class == "backlight"
        };
        wanted.then(|| name.to_owned())
    })
}
