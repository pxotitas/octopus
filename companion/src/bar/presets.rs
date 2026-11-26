//! Whole bars to start from: which modules, in which part of the column.

pub struct Preset {
    pub name: &'static str,
    pub about: &'static str,
    pub top: &'static [&'static str],
    pub center: &'static [&'static str],
    pub bottom: &'static [&'static str],
}

pub const PRESETS: [Preset; 5] = [
    Preset {
        name: "padrão",
        about: "o octopus como vem",
        top: &["logo", "workspaces"],
        center: &["clock"],
        bottom: &["tray", "battery", "controls"],
    },
    Preset {
        name: "mínima",
        about: "só o essencial",
        top: &["workspaces"],
        center: &["clock"],
        bottom: &["battery"],
    },
    Preset {
        name: "monitor",
        about: "cpu, memória e temperatura à vista",
        top: &["logo", "workspaces"],
        center: &["clock"],
        bottom: &["cpu", "memory", "temperature", "tray", "battery", "controls"],
    },
    Preset {
        name: "música",
        about: "tocar, pausar e o volume na mão",
        top: &["logo", "workspaces"],
        center: &["clock", "media"],
        bottom: &["volume", "tray", "battery", "controls"],
    },
    Preset {
        name: "foco",
        about: "a tela acordada e as notificações por perto",
        top: &["logo", "workspaces"],
        center: &["clock"],
        bottom: &["idle", "notifications", "tray", "battery", "power"],
    },
];

pub fn apply(preset: &Preset, bar: &mut crate::config::Bar) {
    let own = |list: &[&str]| list.iter().map(|s| (*s).to_owned()).collect();
    bar.top = own(preset.top);
    bar.center = own(preset.center);
    bar.bottom = own(preset.bottom);
}
