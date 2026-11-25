#!/usr/bin/env python3

"""The weather from Open-Meteo (open-meteo.com): no key, a week back and two
ahead, so the dash can answer "yesterday" or "next thursday".

With no place configured, the place comes from the IP (ipwho.is), which is
only approximate; a named place goes through Open-Meteo's geocoding, and
"lat,lon" is used as is. The place that answered is always reported back.

Any failure is reported as unavailable, never as a guessed value.
"""

import json
import math
import os
import pathlib
import subprocess
import sys
import time
import urllib.parse
from datetime import date, datetime

LANG = os.environ.get("OCTOPUS_LANG", "pt")
TIMEOUT = 12
PAST_DAYS, FUTURE_DAYS = 7, 14
CACHE = pathlib.Path(os.environ.get("XDG_CACHE_HOME", pathlib.Path.home() / ".cache")) / "octopus" / "place.json"
PLACE_TTL = 6 * 3600


class PlaceNotFound(Exception):
    """The name was asked for and the geocoder does not know it."""


# WMO weather codes: (Material Symbols ligature by day, by night, pt, en)
CODES = {
    0: ("clear_day", "clear_night", "céu limpo", "clear sky"),
    1: ("partly_cloudy_day", "partly_cloudy_night", "quase limpo", "mostly clear"),
    2: ("partly_cloudy_day", "partly_cloudy_night", "parcialmente nublado", "partly cloudy"),
    3: ("cloud", "cloud", "nublado", "overcast"),
    45: ("foggy", "foggy", "neblina", "fog"),
    48: ("foggy", "foggy", "neblina com geada", "freezing fog"),
    51: ("rainy_light", "rainy_light", "garoa fraca", "light drizzle"),
    53: ("rainy_light", "rainy_light", "garoa", "drizzle"),
    55: ("rainy_light", "rainy_light", "garoa forte", "heavy drizzle"),
    56: ("rainy_light", "rainy_light", "garoa congelante", "freezing drizzle"),
    57: ("rainy_light", "rainy_light", "garoa congelante forte", "heavy freezing drizzle"),
    61: ("rainy_light", "rainy_light", "chuva fraca", "light rain"),
    63: ("rainy", "rainy", "chuva", "rain"),
    65: ("rainy", "rainy", "chuva forte", "heavy rain"),
    66: ("rainy", "rainy", "chuva congelante", "freezing rain"),
    67: ("rainy", "rainy", "chuva congelante forte", "heavy freezing rain"),
    71: ("weather_snowy", "weather_snowy", "neve fraca", "light snow"),
    73: ("weather_snowy", "weather_snowy", "neve", "snow"),
    75: ("weather_snowy", "weather_snowy", "neve forte", "heavy snow"),
    77: ("weather_snowy", "weather_snowy", "grãos de neve", "snow grains"),
    80: ("rainy_light", "rainy_light", "pancadas fracas", "light showers"),
    81: ("rainy", "rainy", "pancadas de chuva", "showers"),
    82: ("rainy", "rainy", "pancadas fortes", "heavy showers"),
    85: ("weather_snowy", "weather_snowy", "pancadas de neve", "snow showers"),
    86: ("weather_snowy", "weather_snowy", "pancadas fortes de neve", "heavy snow showers"),
    95: ("thunderstorm", "thunderstorm", "trovoada", "thunderstorm"),
    96: ("thunderstorm", "thunderstorm", "trovoada com granizo", "thunderstorm with hail"),
    99: ("thunderstorm", "thunderstorm", "trovoada forte com granizo", "heavy thunderstorm with hail"),
}

MOONS = {
    "pt": ["lua nova", "lua crescente", "quarto crescente", "crescente gibosa",
           "lua cheia", "minguante gibosa", "quarto minguante", "lua minguante"],
    "en": ["new moon", "waxing crescent", "first quarter", "waxing gibbous",
           "full moon", "waning gibbous", "last quarter", "waning crescent"],
}

AQI = {
    "pt": ["boa", "razoável", "moderada", "ruim", "muito ruim", "péssima"],
    "en": ["good", "fair", "moderate", "poor", "very poor", "extremely poor"],
}


def icon(code, day=True):
    entry = CODES.get(code)
    return (entry[0] if day else entry[1]) if entry else "cloud"


def describe(code):
    entry = CODES.get(code)
    return (entry[2] if LANG == "pt" else entry[3]) if entry else ""


def get(url):
    result = subprocess.run(
        ["curl", "-sS", "--fail", "--max-time", str(TIMEOUT), url],
        capture_output=True, text=True,
    )
    if result.returncode != 0 or not result.stdout.strip():
        raise RuntimeError(result.stderr.strip() or f"no answer from {url}")
    return json.loads(result.stdout)


def locate(place):
    """(name, region, latitude, longitude), cached for a while."""
    key = place.strip().lower()
    try:
        cached = json.loads(CACHE.read_text())
        if cached["key"] == key and time.time() - cached["at"] < PLACE_TTL:
            return tuple(cached["place"])
    except (OSError, ValueError, KeyError):
        pass

    if not key:
        data = get("https://ipwho.is/?fields=success,city,country,latitude,longitude")
        if not data.get("success", True):
            raise RuntimeError("ipwho.is could not place this address")
        found = (data["city"], data["country"], data["latitude"], data["longitude"])
    elif "," in key and all(p.strip().lstrip("-").replace(".", "", 1).isdigit() for p in key.split(",", 1)):
        lat, lon = (float(p) for p in key.split(",", 1))
        found = (place.strip(), "", lat, lon)
    else:
        # A name is only matched in the language asked for ("Lisboa" is not
        # "Lisbon"), so both are searched.
        results = []
        for language in dict.fromkeys((LANG, "pt", "en")):
            query = urllib.parse.urlencode({"name": place.strip(), "count": 10, "language": language})
            results += get(f"https://geocoding-api.open-meteo.com/v1/search?{query}").get("results") or []
        if not results:
            raise PlaceNotFound(place)
        # Namesakes come in no useful order; the biggest is the likeliest.
        r = max(results, key=lambda r: r.get("population") or 0)
        found = (r["name"], r.get("country", ""), r["latitude"], r["longitude"])

    CACHE.parent.mkdir(parents=True, exist_ok=True)
    CACHE.write_text(json.dumps({"key": key, "at": time.time(), "place": found}))
    return found


def moon(day):
    """The moon's age as a fraction of its cycle (0 new, 0.5 full)."""
    known_new = date(2000, 1, 6)
    cycle = 29.530588853
    return ((day - known_new).days % cycle) / cycle


def moon_name(fraction):
    return MOONS[LANG][int((fraction * 8) + 0.5) % 8]


def hhmm(iso):
    return iso[11:16] if iso else ""


def forecast(lat, lon):
    query = urllib.parse.urlencode({
        "latitude": lat,
        "longitude": lon,
        "timezone": "auto",
        "past_days": PAST_DAYS,
        "forecast_days": FUTURE_DAYS + 1,
        "wind_speed_unit": "kmh",
        "current": ",".join([
            "temperature_2m", "apparent_temperature", "relative_humidity_2m", "is_day",
            "precipitation", "weather_code", "cloud_cover", "pressure_msl",
            "wind_speed_10m", "wind_direction_10m", "wind_gusts_10m", "uv_index", "visibility",
        ]),
        "hourly": "temperature_2m,weather_code,precipitation_probability,is_day",
        "daily": ",".join([
            "weather_code", "temperature_2m_max", "temperature_2m_min", "precipitation_sum",
            "precipitation_probability_max", "wind_speed_10m_max", "wind_gusts_10m_max",
            "wind_direction_10m_dominant", "uv_index_max", "sunrise", "sunset",
        ]),
    })
    return get(f"https://api.open-meteo.com/v1/forecast?{query}")


def air(lat, lon):
    query = urllib.parse.urlencode({
        "latitude": lat, "longitude": lon, "timezone": "auto",
        "current": "european_aqi,pm2_5,pm10",
    })
    try:
        current = get(f"https://air-quality-api.open-meteo.com/v1/air-quality?{query}")["current"]
    except Exception:
        return None
    index = current.get("european_aqi")
    if index is None:
        return None
    level = min(5, sum(index > edge for edge in (20, 40, 60, 80, 100)))
    return {
        "index": round(index),
        "label": AQI[LANG][level],
        "level": level,
        "pm25": round(current.get("pm2_5") or 0),
        "pm10": round(current.get("pm10") or 0),
    }


def number(value, default=0):
    return round(value) if value is not None else default


def report(place):
    name, region, lat, lon = locate(place)
    data = forecast(lat, lon)
    now = data["current"]
    daily, hourly = data["daily"], data["hourly"]
    today = date.fromisoformat(now["time"][:10])
    this_hour = now["time"][:13]

    hours_by_day = {}
    upcoming = []
    for i, stamp in enumerate(hourly["time"]):
        entry = {
            "hour": int(stamp[11:13]),
            "temperature": number(hourly["temperature_2m"][i]),
            "icon": icon(hourly["weather_code"][i], bool(hourly["is_day"][i])),
            "rain": number(hourly["precipitation_probability"][i]),
        }
        hours_by_day.setdefault(stamp[:10], []).append(entry)
        if this_hour <= stamp[:13] and len(upcoming) < 24:
            upcoming.append({**entry, "tomorrow": date.fromisoformat(stamp[:10]) > today})

    days = []
    for i, stamp in enumerate(daily["time"]):
        day = date.fromisoformat(stamp)
        code = daily["weather_code"][i]
        days.append({
            "date": stamp,
            "offset": (day - today).days,
            "weekday": day.weekday(),
            "icon": icon(code),
            "description": describe(code),
            "high": number(daily["temperature_2m_max"][i]),
            "low": number(daily["temperature_2m_min"][i]),
            "rain": number(daily["precipitation_probability_max"][i]),
            "rainMm": round(daily["precipitation_sum"][i] or 0, 1),
            "wind": number(daily["wind_speed_10m_max"][i]),
            "gusts": number(daily["wind_gusts_10m_max"][i]),
            "windDir": number(daily["wind_direction_10m_dominant"][i]),
            "uv": round(daily["uv_index_max"][i] or 0, 1),
            "sunrise": hhmm(daily["sunrise"][i]),
            "sunset": hhmm(daily["sunset"][i]),
            "moon": round(moon(day), 3),
            "moonName": moon_name(moon(day)),
            "hours": hours_by_day.get(stamp, []),
        })
    current_day = next(d for d in days if d["offset"] == 0)
    code = now["weather_code"]

    return {
        "available": True,
        "place": name,
        "region": region,
        "temperature": number(now["temperature_2m"]),
        "feelsLike": number(now["apparent_temperature"]),
        "description": describe(code),
        "icon": icon(code, bool(now["is_day"])),
        "isDay": bool(now["is_day"]),
        "humidity": number(now["relative_humidity_2m"]),
        "wind": number(now["wind_speed_10m"]),
        "windDir": number(now["wind_direction_10m"]),
        "gusts": number(now["wind_gusts_10m"]),
        "pressure": number(now["pressure_msl"]),
        "uv": round(now.get("uv_index") or 0, 1),
        "visibility": round((now.get("visibility") or 0) / 1000, 1),
        "cloud": number(now["cloud_cover"]),
        "precipitation": round(now.get("precipitation") or 0, 1),
        "low": current_day["low"],
        "high": current_day["high"],
        "sunrise": current_day["sunrise"],
        "sunset": current_day["sunset"],
        "moon": current_day["moon"],
        "moonName": current_day["moonName"],
        "air": air(lat, lon),
        "updated": datetime.now().strftime("%H:%M"),
        "hourly": upcoming,
        "days": days,
    }


if __name__ == "__main__":
    # Every argument is part of the place, so callers need not quote it.
    place = " ".join(sys.argv[1:]).strip()
    try:
        print(json.dumps(report(place), ensure_ascii=False))
    except PlaceNotFound as error:
        # Shown in the settings; on any other failure the shell keeps its
        # last good reading.
        sys.stderr.write(f"weather: no place called {error}\n")
        print(json.dumps({"available": False, "reason": "place"}))
    except Exception as error:  # a card must never take the shell down
        sys.stderr.write(f"weather failed: {error}\n")
        print(json.dumps({"available": False}))
