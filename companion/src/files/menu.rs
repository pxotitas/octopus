//! The keyboard shortcuts and the right-click menus.

use gtk::gio;
use gtk::prelude::*;

use super::{ops::Ops, tags};
use crate::i18n::t;

pub fn accels(app: &gtk::Application) {
    let keys: &[(&str, &[&str])] = &[
        ("win.back", &["<Alt>Left", "BackSpace"]),
        ("win.forward", &["<Alt>Right"]),
        ("win.up", &["<Alt>Up"]),
        ("win.location", &["<Control>l"]),
        ("win.search", &["<Control>f"]),
        ("win.info", &["<Control>i"]),
        ("win.rename", &["F2"]),
        ("win.trash", &["Delete"]),
        ("win.delete", &["<Shift>Delete"]),
        ("win.copy", &["<Control>c"]),
        ("win.cut", &["<Control>x"]),
        ("win.paste", &["<Control>v"]),
        ("win.duplicate", &["<Control>d"]),
        ("win.select-all", &["<Control>a"]),
        ("win.hidden", &["<Control>h"]),
        ("win.new-folder", &["<Control><Shift>n"]),
        ("win.new-tab", &["<Control>t"]),
        ("win.close-tab", &["<Control>w"]),
        ("win.new-window", &["<Control>n"]),
        ("win.undo", &["<Control>z"]),
        ("win.zoom-in", &["<Control>plus", "<Control>equal"]),
        ("win.zoom-out", &["<Control>minus"]),
        ("win.view::icons", &["<Control>1"]),
        ("win.view::list", &["<Control>2"]),
        ("win.view::columns", &["<Control>3"]),
        ("win.bookmark", &["<Control><Shift>d"]),
        ("win.terminal", &["<Control><Alt>t"]),
        ("win.reload", &["F5", "<Control>r"]),
    ];
    for (action, accels) in keys {
        app.set_accels_for_action(action, accels);
    }
}

fn item(menu: &gio::Menu, label: &str, action: &str) {
    menu.append(Some(label), Some(action));
}

/// The menu for the selection (`files` it applies to).
pub fn for_selection(files: &[gio::File], infos: &[gio::FileInfo], in_trash: bool) -> gio::Menu {
    let menu = gio::Menu::new();
    let open = gio::Menu::new();
    item(&open, t("abrir"), "win.open");
    if files.len() == 1 {
        if let Some(ct) = infos.first().and_then(|i| i.content_type()) {
            let with = gio::Menu::new();
            for app in gio::AppInfo::recommended_for_type(&ct).into_iter().take(8) {
                if let Some(id) = app.id() {
                    let entry = gio::MenuItem::new(Some(&app.display_name()), None);
                    entry.set_action_and_target_value(Some("win.open-with"), Some(&id.to_variant()));
                    with.append_item(&entry);
                }
            }
            if with.n_items() > 0 {
                open.append_submenu(Some(t("abrir com")), &with);
            }
        }
    }
    item(&open, t("visualização rápida"), "win.quicklook");
    menu.append_section(None, &open);

    if in_trash {
        let trash = gio::Menu::new();
        item(&trash, t("restaurar"), "win.restore");
        item(&trash, t("apagar pra sempre"), "win.delete");
        menu.append_section(None, &trash);
        return menu;
    }

    let edit = gio::Menu::new();
    item(&edit, t("copiar"), "win.copy");
    item(&edit, t("recortar"), "win.cut");
    item(&edit, t("duplicar"), "win.duplicate");
    item(&edit, t("renomear…"), "win.rename");
    item(&edit, t("copiar caminho"), "win.copy-path");
    menu.append_section(None, &edit);

    let more = gio::Menu::new();
    item(&more, t("compactar"), "win.compress");
    if files.len() == 1 && files[0].basename().is_some_and(|b| Ops::can_extract(&b.to_string_lossy())) {
        item(&more, t("extrair aqui"), "win.extract");
    }
    item(&more, t("enviar por localsend…"), "win.share");
    if infos.len() == 1 && infos[0].file_type() == gio::FileType::Directory {
        item(&more, t("abrir em nova aba"), "win.open-tab");
        item(&more, t("abrir no terminal"), "win.terminal");
        item(&more, t("marcador"), "win.bookmark");
    }
    item(&more, t("obter informações"), "win.info");
    menu.append_section(None, &more);

    let tag_menu = gio::Menu::new();
    for (id, name, _) in tags::COLORS {
        let entry = gio::MenuItem::new(Some(t(name)), None);
        entry.set_action_and_target_value(Some("win.tag"), Some(&id.to_variant()));
        tag_menu.append_item(&entry);
    }
    menu.append_submenu(Some(t("tags")), &tag_menu);

    let danger = gio::Menu::new();
    item(&danger, t("mover pra lixeira"), "win.trash");
    menu.append_section(None, &danger);
    menu
}

/// The menu for the folder's empty space.
pub fn for_folder(in_trash: bool) -> gio::Menu {
    let menu = gio::Menu::new();
    if in_trash {
        item(&menu, t("esvaziar lixeira"), "win.empty-trash");
        return menu;
    }
    let make = gio::Menu::new();
    item(&make, t("nova pasta"), "win.new-folder");
    item(&make, t("colar"), "win.paste");
    item(&make, t("clonar repositório aqui…"), "win.clone");
    menu.append_section(None, &make);
    let view = gio::Menu::new();
    let by = gio::Menu::new();
    for (key, label) in [("name", "nome"), ("modified", "modificado"), ("size", "tamanho"), ("kind", "tipo")] {
        let entry = gio::MenuItem::new(Some(t(label)), None);
        entry.set_action_and_target_value(Some("win.sort"), Some(&key.to_variant()));
        by.append_item(&entry);
    }
    by.append(Some(t("decrescente")), Some("win.descending"));
    view.append_submenu(Some(t("ordenar por")), &by);
    item(&view, t("mostrar ocultos"), "win.hidden");
    menu.append_section(None, &view);
    let place = gio::Menu::new();
    item(&place, t("abrir no terminal"), "win.terminal");
    item(&place, t("marcador"), "win.bookmark");
    item(&place, t("obter informações"), "win.info");
    menu.append_section(None, &place);
    menu
}
