use std::collections::HashMap;
use std::sync::OnceLock;

static ENGLISH: OnceLock<bool> = OnceLock::new();

/// Picks the language once at startup: `pt`, `en`, or `auto` for the
/// system locale. Portuguese is the source text; English is looked up.
pub fn init(language: &str) {
    let english = match language {
        "pt" => false,
        "en" => true,
        _ => !std::env::var("LANG").unwrap_or_default().starts_with("pt"),
    };
    let _ = ENGLISH.set(english);
    std::env::set_var("OCTOPUS_LANG", if english { "en" } else { "pt" });
}

pub fn english() -> bool {
    *ENGLISH.get().unwrap_or(&false)
}

pub fn t(text: &'static str) -> &'static str {
    if !english() {
        return text;
    }
    static TABLE: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    TABLE
        .get_or_init(|| ENTRIES.iter().copied().collect())
        .get(text)
        .copied()
        .unwrap_or(text)
}

/// `t` for text with one `{}` in it.
pub fn tf(text: &'static str, value: impl std::fmt::Display) -> String {
    t(text).replacen("{}", &value.to_string(), 1)
}

pub fn locale() -> chrono::Locale {
    if english() {
        chrono::Locale::en_US
    } else {
        chrono::Locale::pt_BR
    }
}

/// Short date, like "segunda, 23 de novembro".
pub fn date(now: &chrono::DateTime<chrono::Local>) -> String {
    now.format_localized(t("%A, %-d de %B"), locale())
        .to_string()
}

const ENTRIES: &[(&str, &str)] = &[
    ("os ícones crescem embaixo do ponteiro", "icons swell under the pointer"),
    ("magnificação", "magnification"),
    ("tamanho dos ícones", "icon size"),
    ("buscar wallpaper", "search wallpapers"),
    ("widgets", "widgets"),
    ("dock", "dock"),
    ("onde cada coisa fica na barra", "where each thing sits on the bar"),
    ("cores e idioma do shell", "the shell's colors and language"),
    ("o automático segue o wallpaper", "automatic follows the wallpaper"),
    ("reinicia o octopus ao aplicar", "restarts octopus when applied"),
    ("a coluna da waybar", "the waybar column"),
    ("flutuante", "floating"),
    ("encostada", "flush"),
    ("estilo", "style"),
    ("encostada fica colada na borda, com cantos côncavos", "flush sits against the edge, with concave corners"),
    ("lado da tela", "side of the screen"),
    ("em pixels", "in pixels"),
    ("apps fixados e abertos, embaixo da tela", "pinned and open apps, at the bottom of the screen"),
    ("fixa", "fixed"),
    ("esconde sozinha", "autohide"),
    ("inteligente", "smart"),
    ("mostrar a dock", "show the dock"),
    ("comportamento", "behavior"),
    ("fixa ocupa espaço; esconde sozinha aparece na borda; inteligente sai da frente das janelas", "fixed takes space; autohide shows at the edge; smart steps aside for windows"),
    ("clique direito num ícone da dock pra fixar ou desafixar", "right-click a dock icon to pin or unpin it"),
    ("clima, notificações e música", "weather, notifications and music"),
    ("vazio usa o ip", "empty uses your ip"),
    ("segundos na tela", "seconds on screen"),
    ("no dash e na tela de bloqueio", "in the dash and on the lock screen"),
    ("abrir", "open"),
    ("nova janela", "new window"),
    ("fixar", "pin"),
    ("desafixar", "unpin"),
    ("fechar todas", "close all"),
    ("fechar", "close"),
    (
        "buscar apps, arquivos e contas",
        "search apps, files and math",
    ),
    ("calcular", "calculate"),
    ("arquivos", "files"),
    ("ação", "action"),
    ("enter copia o resultado", "enter copies the result"),
    ("sessão", "session"),
    ("digite o nome de um arquivo", "type a file name"),
    ("busca na sua pasta pessoal", "searches your home folder"),
    ("energia", "power"),
    ("wallpapers", "wallpapers"),
    ("wi-fi", "wi-fi"),
    ("tema", "theme"),
    ("suspender", "suspend"),
    ("silenciar", "mute"),
    ("reiniciar", "restart"),
    ("procurar dispositivos", "search for devices"),
    ("notificações", "notifications"),
    ("nada tocando", "nothing playing"),
    ("microfone", "microphone"),
    ("limpar tudo", "clear all"),
    ("falha na autenticação", "authentication failed"),
    ("escuro", "dark"),
    ("desligar", "shut down"),
    ("desligado", "off"),
    ("configurações", "settings"),
    ("claro", "light"),
    ("área de transferência", "clipboard"),
    ("voltar", "back"),
    ("visão geral", "overview"),
    ("verificando…", "checking…"),
    ("tudo em dia", "all caught up"),
    ("toque pra parear", "tap to pair"),
    ("topo", "top"),
    ("tocar", "play"),
    ("subir", "move up"),
    ("senha inválida", "invalid password"),
    ("senha incorreta", "wrong password"),
    ("sem letra pra essa música", "no lyrics for this song"),
    ("sem dados", "no data"),
    ("sem dados do clima", "no weather data"),
    ("saída", "output"),
    ("salva", "saved"),
    ("sair", "log out"),
    ("próximas horas", "next hours"),
    ("próxima", "next"),
    ("procurar redes", "search networks"),
    ("procurando…", "searching…"),
    ("procurando a letra…", "looking for lyrics…"),
    ("posição", "position"),
    ("pareado", "paired"),
    ("outros", "other"),
    ("oculto", "hidden"),
    ("o wi-fi está desligado", "wi-fi is off"),
    ("o bluetooth está desligado", "bluetooth is off"),
    ("não perturbe", "do not disturb"),
    ("não deu pra buscar a letra", "couldn't fetch the lyrics"),
    ("notificações (s)", "notifications (s)"),
    ("nenhum app na bandeja", "no apps in the tray"),
    ("nenhum adaptador bluetooth", "no bluetooth adapter"),
    ("mídia", "media"),
    ("mudo", "muted"),
    ("mixer", "mixer"),
    ("meio", "middle"),
    ("ligado", "on"),
    ("letras", "lyrics"),
    (
        "letras desligadas no octopus.toml",
        "lyrics are off in octopus.toml",
    ),
    ("largura", "width"),
    ("instrumental", "instrumental"),
    ("esquerda", "left"),
    ("dispensar", "dismiss"),
    ("direita", "right"),
    ("digite sua senha", "enter your password"),
    ("desconectado", "disconnected"),
    ("descer", "move down"),
    ("conectar", "connect"),
    ("conectando…", "connecting…"),
    ("conectado", "connected"),
    (
        "clima, notificações e letras valem depois de reiniciar o octopus",
        "weather, notifications, lyrics and language apply after restarting octopus",
    ),
    ("clima", "weather"),
    ("buscar", "search"),
    ("bloquear", "lock"),
    ("base", "bottom"),
    ("barra", "bar"),
    ("bandeja", "tray"),
    ("ativo", "active"),
    ("aplicar", "apply"),
    ("aparência", "appearance"),
    ("apagar tudo", "delete all"),
    ("anterior", "previous"),
    ("aguarde…", "wait…"),
    (
        "conectada · toque pra desconectar",
        "connected · tap to disconnect",
    ),
    ("cidade do clima", "weather city"),
    ("pelo ip", "by ip"),
    ("widgets da barra", "bar widgets"),
    ("central de controle", "control center"),
    ("relógio", "clock"),
    ("logo do arch", "arch logo"),
    ("controles", "controls"),
    ("bateria", "battery"),
    ("workspaces", "workspaces"),
    ("emoji", "emoji"),
    ("automático", "automatic"),
    ("modo de cores", "color mode"),
    ("idioma", "language"),
    ("sistema", "system"),
    ("português", "português"),
    ("english", "english"),
    ("imagem", "image"),
    ("senha de {}", "password for {}"),
    ("sensação {}°", "feels like {}°"),
    ("umidade {}%", "humidity {}%"),
    ("vento {} km/h", "wind {} km/h"),
    ("mín {}° · máx {}°", "low {}° · high {}°"),
    ("aperte de novo pra {}", "press again to {}"),
    ("{} na bandeja", "{} in the tray"),
    ("%A, %-d de %B", "%A, %B %-d"),
    ("buscar emoji", "search emoji"),
    ("nenhum emoji", "no emoji"),
    ("segue o wallpaper", "follows the wallpaper"),
];
