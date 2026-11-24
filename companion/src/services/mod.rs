//! The system behind the panels; each service owns its state in a Store the UI subscribes to.

pub mod audio;
pub mod bluetooth;
pub mod brightness;
pub mod cava;
pub mod lyrics;
pub mod media;
pub mod network;
pub mod niri;
pub mod notifications;
pub mod theme;
pub mod tray;
pub mod weather;

use std::rc::Rc;

use crate::config::Config;

pub struct Services {
    pub audio: Rc<audio::Audio>,
    pub screen: Rc<brightness::Light>,
    pub keyboard: Rc<brightness::Light>,
    pub wifi: Rc<network::Wifi>,
    pub bluetooth: Rc<bluetooth::Bluetooth>,
    pub media: Rc<media::Media>,
    pub weather: Rc<weather::Weather>,
    pub cava: Rc<cava::Cava>,
    pub notifications: Rc<notifications::Notifications>,
    pub theme: Rc<theme::Theme>,
    pub tray: Rc<tray::Tray>,
}

impl Services {
    pub fn new(config: &Config) -> Rc<Self> {
        Rc::new(Self {
            audio: audio::Audio::new(),
            screen: brightness::Light::new(&config.brightness.screen, false),
            keyboard: brightness::Light::new(&config.brightness.keyboard, true),
            wifi: network::Wifi::new(),
            bluetooth: bluetooth::Bluetooth::new(),
            media: media::Media::new(),
            weather: weather::Weather::new(&config.weather.place),
            cava: cava::Cava::new(),
            notifications: notifications::Notifications::new(),
            theme: theme::Theme::new(),
            tray: tray::Tray::new(),
        })
    }
}
