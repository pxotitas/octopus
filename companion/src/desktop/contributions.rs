//! A github user's contribution calendar on the desktop: the little squares
//! from the profile, as many weeks as the widget is wide.

use std::cell::{Cell, RefCell};

use chrono::Datelike;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene, gsk};

use crate::i18n::{t, tf};
use crate::icons;
use crate::services::contributions::{totals, Contributions, Day};
use crate::widgets::shape::{self, Kind, Shape};

/// github's own greens, light and dark.
const GREEN_LIGHT: [u32; 5] = [0xebedf0, 0x9be9a8, 0x40c463, 0x30a14e, 0x216e39];
const GREEN_DARK: [u32; 5] = [0x2a2e35, 0x0e4429, 0x006d32, 0x26a641, 0x39d353];

fn rgb(hex: u32) -> gdk::RGBA {
    gdk::RGBA::new(
        ((hex >> 16) & 0xff) as f32 / 255.0,
        ((hex >> 8) & 0xff) as f32 / 255.0,
        (hex & 0xff) as f32 / 255.0,
        1.0,
    )
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Grid {
        /// One column per week, sunday on top, like github.
        pub weeks: RefCell<Vec<[Option<Day>; 7]>>,
        /// github's greens, or the theme's primary in steps.
        pub green: Cell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Grid {
        const NAME: &'static str = "OctopusContributionGrid";
        type Type = super::Grid;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("contributions");
        }
    }

    impl ObjectImpl for Grid {}

    impl WidgetImpl for Grid {
        fn measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            if orientation == gtk::Orientation::Horizontal {
                (120, 420, -1, -1)
            } else {
                (70, 110, -1, -1)
            }
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let (w, h) = (widget.width() as f32, widget.height() as f32);
            let weeks = self.weeks.borrow();
            if w <= 0.0 || h <= 0.0 || weeks.is_empty() {
                return;
            }
            let (pitch, count) = super::fit(w, h, weeks.len());
            let cell = pitch * 0.82;
            let radius = cell * 0.3;
            let ink = widget.color();
            let dark = ink.red() + ink.green() + ink.blue() > 1.5;
            let palette: Vec<gdk::RGBA> = if self.green.get() {
                (if dark { GREEN_DARK } else { GREEN_LIGHT }).iter().map(|&c| rgb(c)).collect()
            } else {
                [0.12, 0.35, 0.55, 0.78, 1.0]
                    .iter()
                    .map(|&a| gdk::RGBA::new(ink.red(), ink.green(), ink.blue(), a))
                    .collect()
            };
            let today = chrono::Local::now().date_naive();
            let left = w - count as f32 * pitch;
            for (col, week) in weeks[weeks.len() - count..].iter().enumerate() {
                for (row, day) in week.iter().enumerate() {
                    let Some(day) = day else { continue };
                    let (x, y) = (left + col as f32 * pitch, row as f32 * pitch);
                    let color = &palette[(day.level as usize).min(4)];
                    if day.date == today {
                        // today is a little cookie
                        snapshot.save();
                        snapshot.translate(&graphene::Point::new(x - pitch * 0.05, y - pitch * 0.05));
                        let path = shape::path(Kind::Cookie4, cell + pitch * 0.1, cell + pitch * 0.1, 0.0);
                        snapshot.append_fill(&path, gsk::FillRule::Winding, color);
                        snapshot.restore();
                        continue;
                    }
                    let rect = gsk::RoundedRect::from_rect(graphene::Rect::new(x, y, cell, cell), radius);
                    let builder = gsk::PathBuilder::new();
                    builder.add_rounded_rect(&rect);
                    snapshot.append_fill(&builder.to_path(), gsk::FillRule::Winding, color);
                }
            }
        }
    }
}

glib::wrapper! {
    pub struct Grid(ObjectSubclass<imp::Grid>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

/// The pitch of one square and how many weeks fit in `w`×`h`.
fn fit(w: f32, h: f32, weeks: usize) -> (f32, usize) {
    let pitch = (h / 7.0).max(4.0);
    let count = ((w / pitch).floor() as usize).clamp(1, weeks.max(1));
    (pitch, count)
}

impl Grid {
    fn new(green: bool) -> Self {
        let grid: Self = glib::Object::new();
        grid.imp().green.set(green);
        grid.set_hexpand(true);
        grid.set_vexpand(true);
        grid.set_has_tooltip(true);
        grid.connect_query_tooltip(|grid, x, y, _, tooltip| {
            let Some(day) = grid.day_at(x as f32, y as f32) else { return false };
            let when = day.date.format_localized(t("%-d de %B"), crate::i18n::locale()).to_string();
            let text = match day.count {
                0 => tf("nenhuma contribuição em {}", when),
                1 => tf("1 contribuição em {}", when),
                n => tf("{} contribuições em ", n) + &when,
            };
            tooltip.set_text(Some(&text));
            true
        });
        grid
    }

    fn set_days(&self, days: &[Day]) {
        let mut weeks: Vec<[Option<Day>; 7]> = Vec::new();
        let today = chrono::Local::now().date_naive();
        for day in days.iter().filter(|d| d.date <= today) {
            let row = day.date.weekday().num_days_from_sunday() as usize;
            if row == 0 || weeks.is_empty() {
                weeks.push(Default::default());
            }
            if let Some(week) = weeks.last_mut() {
                week[row] = Some(day.clone());
            }
        }
        *self.imp().weeks.borrow_mut() = weeks;
        self.queue_draw();
    }

    fn day_at(&self, x: f32, y: f32) -> Option<Day> {
        let weeks = self.imp().weeks.borrow();
        let (w, h) = (self.width() as f32, self.height() as f32);
        let (pitch, count) = fit(w, h, weeks.len());
        let left = w - count as f32 * pitch;
        if x < left || y < 0.0 {
            return None;
        }
        let (col, row) = (((x - left) / pitch) as usize, (y / pitch) as usize);
        if row > 6 || col >= count {
            return None;
        }
        weeks[weeks.len() - count + col][row].clone()
    }
}

pub fn build(user: &str, green: bool) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 12);
    root.add_css_class("desktop-widget");
    root.add_css_class("desktop-contributions");

    let head = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let badge = Shape::new(Kind::SoftBurst, 40, true, Some(&icons::icon("commit")));
    badge.add_css_class("desktop-github-badge");
    head.append(&badge);
    let titles = gtk::Box::new(gtk::Orientation::Vertical, 0);
    titles.set_valign(gtk::Align::Center);
    titles.set_hexpand(true);
    let title = gtk::Label::new(Some(t("contribuições")));
    title.add_css_class("desktop-title");
    title.set_xalign(0.0);
    let who = gtk::Label::new(Some(&format!("@{user}")));
    who.add_css_class("dim");
    who.set_xalign(0.0);
    titles.append(&title);
    titles.append(&who);
    head.append(&titles);
    let stats = gtk::Box::new(gtk::Orientation::Vertical, 0);
    stats.set_valign(gtk::Align::Center);
    let total = gtk::Label::new(None);
    total.add_css_class("contrib-total");
    total.set_xalign(1.0);
    let streak = gtk::Label::new(None);
    streak.add_css_class("dim");
    streak.set_xalign(1.0);
    stats.append(&total);
    stats.append(&streak);
    head.append(&stats);
    root.append(&head);

    let grid = Grid::new(green);
    grid.add_css_class("contrib-grid");
    root.append(&grid);

    let service = Contributions::new(user);
    let keep = service.clone();
    service.days.subscribe(move |days| {
        let _ = &keep;
        let Some(days) = days else {
            total.set_label(t("buscando…"));
            return;
        };
        grid.set_days(days);
        let (sum, run) = totals(days);
        total.set_label(&tf("{} no último ano", sum));
        streak.set_label(&match run {
            0 => t("sem sequência").to_owned(),
            1 => t("1 dia seguido").to_owned(),
            n => tf("{} dias seguidos", n),
        });
    });
    root
}
