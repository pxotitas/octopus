"""The media page: the full video, the clips, the screenshots and the brand."""

CLIPS = [
    ("dash", {"en": ("dash", "overview, then the music with its synced lyrics"), "pt": ("dash", "visão geral, depois a música com a letra sincronizada")}),
    ("calendar", {"en": ("calendar", "your events in the dash, a day at a time"), "pt": ("calendário", "seus eventos no dash, um dia por vez")}),
    ("weather", {"en": ("weather and wallpapers", "asking the weather, then the wallpaper picker"), "pt": ("clima e wallpapers", "perguntando o clima, depois o seletor de wallpaper")}),
    ("control-center", {"en": ("control center", "quick toggles, sliders and the player"), "pt": ("central de controle", "atalhos, sliders e o player")}),
    ("widgets", {"en": ("arranging widgets", "the right-click menu, moving one, and the case"), "pt": ("organizando widgets", "o menu do clique direito, movendo um, e a maleta")}),
    ("settings", {"en": ("bar editor", "the settings tree, module presets and the catalog"), "pt": ("editor da barra", "a árvore das configurações, presets de módulo e o catálogo")}),
    ("maker", {"en": ("making a module", "an icon, a command, and trying it"), "pt": ("criando um módulo", "um ícone, um comando, e testando")}),
    ("palette", {"en": ("palette from the wallpaper", "a new wallpaper repaints the shell, then back"), "pt": ("paleta do wallpaper", "um wallpaper novo repinta o shell, e volta")}),
    ("launchers", {"en": ("launchers", "the spotlight and rofi"), "pt": ("launchers", "o spotlight e o rofi")}),
    ("calculator", {"en": ("spotlight math", "sums typed after <code>=</code>"), "pt": ("conta no spotlight", "contas depois do <code>=</code>")}),
    ("emoji", {"en": ("emoji", "the picker, with search"), "pt": ("emoji", "o seletor, com busca")}),
    ("lock", {"en": ("lock screen", "shapes that change at every key"), "pt": ("tela de bloqueio", "shapes que mudam a cada tecla")}),
]

GROUPS = [
    ("shots-apps", "cookie9", {"en": "apps", "pt": "apps"}, [
        ("files", {"en": "octopus files, as icons", "pt": "octopus files, em ícones"}),
        ("files-columns", {"en": "octopus files in columns, in a git repository", "pt": "octopus files em colunas, num repositório git"}),
        ("reminders", {"en": "the reminders app", "pt": "o app de lembretes"}),
        ("launcher-remind", {"en": "a reminder from the spotlight", "pt": "um lembrete pelo spotlight"}),
    ]),
    ("shots-shell", "cookie9", {"en": "the shell", "pt": "o shell"}, [
        ("desktop", {"en": "the bar and the desktop widgets", "pt": "a barra e os widgets de desktop"}),
        ("desktop-menu", {"en": "right-click on the desktop", "pt": "clique direito na área de trabalho"}),
        ("desktop-arrange", {"en": "arranging widgets, with the case", "pt": "organizando os widgets, com a maleta"}),
        ("desktop-widgets", {"en": "photo, note, agenda, system, battery and a digital clock", "pt": "foto, nota, agenda, sistema, bateria e um relógio digital"}),
        ("dash", {"en": "dash, with your calendar's events", "pt": "dash, com os eventos da agenda"}),
        ("control-center", {"en": "control center", "pt": "central de controle"}),
        ("notification", {"en": "a notification", "pt": "uma notificação"}),
        ("osd", {"en": "volume osd", "pt": "osd de volume"}),
        ("wallpapers", {"en": "wallpaper picker", "pt": "seletor de wallpaper"}),
    ]),
    ("shots-weather", "sunny", {"en": "the weather", "pt": "o clima"}, [
        ("weather", {"en": "right now", "pt": "agora"}),
        ("weather-tomorrow", {"en": "> tomorrow", "pt": "> amanhã"}),
        ("weather-yesterday", {"en": "> yesterday", "pt": "> ontem"}),
        ("weather-week", {"en": "> week", "pt": "> semana"}),
        ("weather-weekend", {"en": "> weekend", "pt": "> fim de semana"}),
    ]),
    ("shots-menus", "clover4", {"en": "menus", "pt": "menus"}, [
        ("wifi", {"en": "wi-fi", "pt": "wi-fi"}),
        ("bluetooth", {"en": "bluetooth", "pt": "bluetooth"}),
        ("audio", {"en": "sound", "pt": "som"}),
        ("tray", {"en": "tray", "pt": "bandeja"}),
        ("power", {"en": "power", "pt": "energia"}),
    ]),
    ("shots-launchers", "softburst", {"en": "launchers and tools", "pt": "launchers e ferramentas"}, [
        ("launcher", {"en": "spotlight", "pt": "spotlight"}),
        ("rofi", {"en": "rofi", "pt": "rofi"}),
        ("clipboard", {"en": "clipboard", "pt": "clipboard"}),
        ("emoji", {"en": "emoji", "pt": "emoji"}),
        ("settings-editor", {"en": "settings: the bar editor", "pt": "configurações: o editor da barra"}),
        ("settings-catalog", {"en": "settings: the catalog of modules", "pt": "configurações: o catálogo de módulos"}),
        ("settings-maker", {"en": "settings: making a module", "pt": "configurações: criando um módulo"}),
        ("settings-widgets", {"en": "settings: each widget's options", "pt": "configurações: as opções de cada widget"}),
        ("settings-calendar", {"en": "settings: your calendar", "pt": "configurações: sua agenda"}),
        ("settings-presets", {"en": "settings: bar presets", "pt": "configurações: presets da barra"}),
        ("settings-colors", {"en": "settings: colors", "pt": "configurações: cores"}),
    ]),
]

TEXT = {
    "en": {
        "title": "media", "lead": "octopus standing still and in motion. the screenshots are in english.",
        "video": "the whole tour", "video_lead": "two minutes and change, without sound: every clip below, one after the other.",
        "clips": "clips", "clips_lead": "the same tour, a piece at a time. they play as you scroll; click one to see it bigger.",
        "shots": "screenshots", "shots_lead": "click one to see it full size.",
        "brand": "logo and banner", "brand_lead": "for posts, videos or a link to the project.",
        "download": "download", "close": "close",
    },
    "pt": {
        "title": "mídia", "lead": "o octopus parado e em movimento. as screenshots estão em inglês.",
        "video": "o tour inteiro", "video_lead": "pouco mais de dois minutos, sem som: todos os clipes abaixo, um depois do outro.",
        "clips": "clipes", "clips_lead": "o mesmo tour, um pedaço por vez. eles tocam conforme você rola; clica num pra ver maior.",
        "shots": "screenshots", "shots_lead": "clica numa pra ver em tamanho real.",
        "brand": "logo e banner", "brand_lead": "pra posts, vídeos ou um link pro projeto.",
        "download": "baixar", "close": "fechar",
    },
}


def render(lang, rel):
    t = TEXT[lang]
    a = f"{rel}assets"
    clips = "\n".join(
        f"""  <figure class="clip" id="{name}">
    <button type="button" data-video="{a}/clips/hq/{name}.mp4" data-label="{text[lang][0]}">
      <video src="{a}/clips/{name}.mp4" poster="{a}/clips/{name}.webp" muted loop playsinline preload="none"></video>
    </button>
    <figcaption><strong>{text[lang][0]}</strong> {text[lang][1]}</figcaption>
  </figure>"""
        for name, text in CLIPS
    )
    groups = []
    for anchor, shape, heading, shots in GROUPS:
        buttons = "\n".join(
            f"""  <button class="shot" type="button" data-full="{a}/screenshots/{name}.webp" data-label="{label[lang]}">
    <img src="{a}/screenshots/thumbs/{name}.webp" alt="{label[lang]}" loading="lazy">
    <span>{label[lang]}</span>
  </button>"""
            for name, label in shots
        )
        groups.append(
            f"""<h3 class="group" id="{anchor}"><span class="shape-badge small tone-b" data-shape="{shape}"></span>{heading[lang]}</h3>
<div class="shots">
{buttons}
</div>"""
        )
    shots = "\n".join(groups)
    return f"""<section class="page-head">
  <h1>{t["title"]}</h1>
  <p class="lead">{t["lead"]}</p>
</section>

<h2 id="video">{t["video"]}</h2>
<p class="lead">{t["video_lead"]}</p>
<video class="tour" src="{a}/video/showcase.mp4" poster="{a}/video/showcase.webp" controls preload="metadata" playsinline></video>

<h2 id="clips">{t["clips"]}</h2>
<p class="lead">{t["clips_lead"]}</p>
<div class="clips">
{clips}
</div>

<h2 id="screenshots">{t["shots"]}</h2>
<p class="lead">{t["shots_lead"]}</p>
{shots}

<h2 id="brand">{t["brand"]}</h2>
<p class="lead">{t["brand_lead"]}</p>
<div class="brand-assets">
  <figure>
    <div class="logo-pair">
      <img src="{a}/logo.svg" alt="" width="120" height="120">
      <img src="{a}/logo-dark.svg" alt="" width="120" height="120">
    </div>
    <figcaption>logo · <a href="{a}/logo.svg" download>svg</a> · <a href="{a}/logo-dark.svg" download>svg ({'dark' if lang == 'en' else 'escuro'})</a></figcaption>
  </figure>
  <figure>
    <img src="{a}/banner.png" alt="" loading="lazy">
    <figcaption>banner · <a href="{a}/banner.png" download>png, 1280×640</a></figcaption>
  </figure>
</div>

<dialog class="viewer">
  <form method="dialog"><button class="close" aria-label="{t["close"]}"><svg viewBox="0 0 24 24" width="24" height="24"><path d="M6.4 19 5 17.6 10.6 12 5 6.4 6.4 5 12 10.6 17.6 5 19 6.4 13.4 12 19 17.6 17.6 19 12 13.4Z"/></svg></button></form>
  <img alt="">
  <video muted loop playsinline controls></video>
  <p></p>
</dialog>"""
