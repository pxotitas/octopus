use std::cell::{Ref, RefCell};
use std::rc::Rc;

/// Callbacks run in registration order.
type Subscribers<T> = RefCell<Vec<Box<dyn Fn(&T)>>>;
pub type Listeners<A> = RefCell<Vec<Box<dyn Fn(A)>>>;

/// A value the UI follows: every subscriber runs once on subscribe and again
/// on each change.
pub struct Store<T> {
    value: RefCell<T>,
    subscribers: Subscribers<T>,
}

impl<T: 'static> Store<T> {
    pub fn new(value: T) -> Rc<Self> {
        Rc::new(Self {
            value: RefCell::new(value),
            subscribers: RefCell::new(Vec::new()),
        })
    }

    pub fn get(&self) -> Ref<'_, T> {
        self.value.borrow()
    }

    pub fn set(&self, value: T) {
        *self.value.borrow_mut() = value;
        self.notify();
    }

    pub fn update(&self, change: impl FnOnce(&mut T)) {
        change(&mut self.value.borrow_mut());
        self.notify();
    }

    pub fn subscribe(&self, subscriber: impl Fn(&T) + 'static) {
        subscriber(&self.value.borrow());
        self.subscribers.borrow_mut().push(Box::new(subscriber));
    }

    fn notify(&self) {
        let value = self.value.borrow();
        for subscriber in self.subscribers.borrow().iter() {
            subscriber(&value);
        }
    }
}
