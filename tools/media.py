"""The media page: the full video, the clips, the screenshots and the brand."""

CLIPS = [
    ("dash", {"en": ("dash", "overview, music and the synced lyrics"), "pt": ("dash", "visão geral, música e a letra sincronizada")}),
    ("weather", {"en": ("weather and wallpapers", "the forecast, then the wallpaper picker"), "pt": ("clima e wallpapers", "a previsão, depois o seletor de wallpaper")}),
    ("control-center", {"en": ("control center", "quick toggles, sliders and the player"), "pt": ("central de controle", "atalhos, sliders e o player")}),
    ("lock", {"en": ("lock screen", "shapes that change at every key"), "pt": ("tela de bloqueio", "shapes que mudam a cada tecla")}),
    ("launchers", {"en": ("launchers", "rofi and the spotlight"), "pt": ("launchers", "o rofi e o spotlight")}),
    ("emoji", {"en": ("emoji", "the picker, with search"), "pt": ("emoji", "o seletor, com busca")}),
    ("settings", {"en": ("settings", "colors, language, bar, widgets and dock"), "pt": ("configurações", "cores, idioma, barra, widgets e dock")}),
    ("calculator", {"en": ("spotlight math", "a sum typed after <code>=</code>"), "pt": ("conta no spotlight", "uma soma depois do <code>=</code>")}),
    ("palette", {"en": ("palette from the wallpaper", "a new wallpaper repaints the shell"), "pt": ("paleta do wallpaper", "um wallpaper novo repinta o shell")}),
]

GROUPS = [
    ("shots-shell", "cookie9", {"en": "the shell", "pt": "o shell"}, [
        ("desktop", {"en": "the bar and the desktop widgets", "pt": "a barra e os widgets de desktop"}),
        ("dash", {"en": "dash", "pt": "dash"}),
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
        ("settings", {"en": "settings", "pt": "configurações"}),
    ]),
]

TEXT = {
    "en": {
        "title": "media", "lead": "octopus standing still and in motion. the screenshots are in english.",
        "video": "the whole tour", "video_lead": "a minute and a half, with sound, recorded before the navigation rail and the new weather tab.",
        "clips": "clips", "clips_lead": "the same tour, a piece at a time. they play as you scroll; click one to see it bigger. recorded before the navigation rail and the new weather tab.",
        "shots": "screenshots", "shots_lead": "click one to see it full size.",
        "brand": "logo and banner", "brand_lead": "for posts, videos or a link to the project.",
        "download": "download", "close": "close",
    },
    "pt": {
        "title": "mídia", "lead": "o octopus parado e em movimento. as screenshots estão em inglês.",
        "video": "o tour inteiro", "video_lead": "um minuto e meio, com som, gravado antes do rail de navegação e da aba de clima nova.",
        "clips": "clipes", "clips_lead": "o mesmo tour, um pedaço por vez. eles tocam conforme você rola; clica num pra ver maior. gravados antes do rail de navegação e da aba de clima nova.",
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
