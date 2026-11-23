use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::exec;
use crate::store::Store;

pub const BARS: usize = 16;

/// cava runs only while something on screen draws it.
pub struct Cava {
    pub bars: Rc<Store<Vec<f64>>>,
    watchers: Cell<u32>,
    process: RefCell<Option<exec::Stream>>,
}

impl Cava {
    pub fn new() -> Rc<Self> {
        Rc::new(Self {
            bars: Store::new(vec![0.0; BARS]),
            watchers: Cell::new(0),
            process: RefCell::new(None),
        })
    }

    pub fn watch(&self, on: bool) {
        let count = if on {
            self.watchers.get() + 1
        } else {
            self.watchers.get().saturating_sub(1)
        };
        self.watchers.set(count);
        if count > 0 && self.process.borrow().is_none() {
            let config = exec::config_dir().join("cava.conf");
            let bars = self.bars.clone();
            let stream = exec::stream("cava", &["-p", &config.to_string_lossy()], move |line| {
                let values: Vec<f64> = line
                    .split(';')
                    .filter(|v| !v.is_empty())
                    .filter_map(|v| v.parse::<f64>().ok())
                    .map(|v| (v / 100.0).clamp(0.0, 1.0))
                    .collect();
                if values.len() == BARS {
                    bars.set(values);
                }
            });
            *self.process.borrow_mut() = stream;
        } else if count == 0 {
            self.process.borrow_mut().take();
            self.bars.set(vec![0.0; BARS]);
        }
    }
}
