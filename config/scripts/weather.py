#!/usr/bin/env python3

# ╭──────────────────────────────────────────────────────────────────────────╮
# │                                                                          │
# │   W E A T H E R                                                          │
# │   current weather and forecast                                           │
# │                                                                          │
# │   github.com/pxdritz1/velatura                                      │
# │                                                                          │
# ╰──────────────────────────────────────────────────────────────────────────╯

"""One wttr.in reading, reduced to what the weather card shows.

With no place configured, wttr.in locates the request by IP, which needs no
key but is only approximate. A named place (a city, a postcode, an airport
code, a `~landmark`) is sent as is. The place that answered is always
reported back.

Any failure is reported as unavailable, never as a guessed value.
"""

import json
import subprocess
import sys
import time
import urllib.parse

ENDPOINT = "https://wttr.in/{place}?format=j1&lang=pt"
TIMEOUT = 12


class PlaceNotFound(Exception):
    """The name was asked for and wttr.in does not know it."""

# wttr.in's condition codes, folded into the handful of shapes worth drawing,
# as Material Symbols ligatures (day, night).
ICONS = [
    ({113}, "clear_day", "clear_night"),
    ({116}, "partly_cloudy_day", "partly_cloudy_night"),
    ({119, 122}, "cloud", "cloud"),
    ({143, 248, 260}, "foggy", "foggy"),
    ({176, 263, 266, 293, 296, 353}, "rainy_light", "rainy_light"),
    ({299, 302, 305, 308, 356, 359}, "rainy", "rainy"),
    ({179, 227, 230, 323, 326, 329, 332, 335, 338, 368, 371}, "weather_snowy", "weather_snowy"),
    ({200, 386, 389, 392, 395}, "thunderstorm", "thunderstorm"),
]

FALLBACK = "cloud"


def icon(code, daytime):
    for codes, day, night in ICONS:
        if code in codes:
            return day if daytime else night
    return FALLBACK


def is_day(hour):
    return 6 <= hour < 18


def description(condition):
    """The Portuguese description when wttr.in has one."""
    for key in ("lang_pt", "weatherDesc"):
        values = condition.get(key) or []
        if values and values[0].get("value", "").strip():
            return values[0]["value"].strip()
    return ""


def fetch(place):
    # Quoted: place names carry spaces and non-ASCII characters.
    url = ENDPOINT.format(place=urllib.parse.quote(place.strip(), safe=""))
    result = subprocess.run(
        ["curl", "-sS", "--max-time", str(TIMEOUT), url],
        capture_output=True, text=True,
    )
    if result.returncode != 0 or not result.stdout.strip():
        raise RuntimeError(result.stderr.strip() or "no answer from wttr.in")
    # An unknown place comes back as 200 with plain text instead of JSON; it
    # is reported apart from a network failure so the settings can show it.
    if not result.stdout.lstrip().startswith("{"):
        raise PlaceNotFound(result.stdout.strip()[:120])
    return json.loads(result.stdout)


def report(place):
    data = fetch(place)
    current = data["current_condition"][0]
    area = data["nearest_area"][0]
    today = data["weather"][0]

    code = int(current["weatherCode"])
    daytime = is_day(time.localtime().tm_hour)

    return {
        "available": True,
        "place": area["areaName"][0]["value"],
        "region": area["country"][0]["value"],
        "temperature": int(current["temp_C"]),
        "feelsLike": int(current["FeelsLikeC"]),
        "description": description(current),
        "icon": icon(code, daytime),
        "humidity": int(current["humidity"]),
        "wind": int(current["windspeedKmph"]),
        "low": int(today["mintempC"]),
        "high": int(today["maxtempC"]),
        # Today and tomorrow, in wttr.in's three-hour steps, so the evening
        # still has a forecast.
        "hourly": [
            {
                "hour": int(block["time"]) // 100,
                "temperature": int(block["tempC"]),
                "icon": icon(int(block["weatherCode"]), is_day(int(block["time"]) // 100)),
                "rain": int(block["chanceofrain"]),
                "tomorrow": day > 0,
            }
            for day, blocks in enumerate(w["hourly"] for w in data["weather"][:2])
            for block in blocks
        ],
    }


if __name__ == "__main__":
    # Every argument is part of the place, so callers need not quote it.
    place = " ".join(sys.argv[1:]).strip()
    try:
        print(json.dumps(report(place)))
    except PlaceNotFound as error:
        # Shown in the settings; on any other failure the shell keeps its
        # last good reading.
        sys.stderr.write(f"weather: {error}\n")
        print(json.dumps({"available": False, "reason": "place"}))
    except Exception as error:  # a card must never take the shell down
        sys.stderr.write(f"weather failed: {error}\n")
        print(json.dumps({"available": False}))
