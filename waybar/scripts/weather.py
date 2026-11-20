#!/usr/bin/env python3
"""Small, fixed-location Open-Meteo reader for the Octopus Waybar."""

import json
import sys
from urllib.error import URLError
from urllib.request import Request, urlopen

LATITUDE = -20.4697
LONGITUDE = -54.6201
LOCATION = "Campo Grande, MS"
URL = (
    "https://api.open-meteo.com/v1/forecast"
    f"?latitude={LATITUDE}&longitude={LONGITUDE}"
    "&current=temperature_2m,weather_code,is_day"
    "&temperature_unit=celsius&timezone=America%2FCampo_Grande"
)

WEATHER = {
    "clear-day": ("󰖙", "Clear"),
    "clear-night": ("󰖔", "Clear night"),
    "partly-cloudy": ("󰖕", "Partly cloudy"),
    "cloudy": ("󰖐", "Cloudy"),
    "fog": ("󰖑", "Fog"),
    "drizzle": ("󰖗", "Drizzle"),
    "rain": ("󰖗", "Rain"),
    "snow": ("󰖘", "Snow"),
    "showers": ("󰖖", "Showers"),
    "thunderstorm": ("󰖓", "Thunderstorm"),
}


def condition(code: int, is_day: bool) -> tuple[str, str]:
    if code == 0:
        return WEATHER["clear-day" if is_day else "clear-night"]
    if code == 1:
        return WEATHER["partly-cloudy"]
    if code == 2:
        return WEATHER["partly-cloudy"]
    if code == 3:
        return WEATHER["cloudy"]
    if code in (45, 48):
        return WEATHER["fog"]
    if code in (51, 53, 55, 56, 57):
        return WEATHER["drizzle"]
    if code in (61, 63, 65, 66, 67):
        return WEATHER["rain"]
    if code in (71, 73, 75, 77):
        return WEATHER["snow"]
    if code in (80, 81, 82, 85, 86):
        return WEATHER["showers"]
    if code in (95, 96, 99):
        return WEATHER["thunderstorm"]
    return WEATHER["cloudy"]


def main() -> None:
    request = Request(URL, headers={"User-Agent": "Octopus-Waybar/1.0"})
    try:
        with urlopen(request, timeout=8) as response:
            data = json.load(response)
        current = data["current"]
        icon, description = condition(
            int(current["weather_code"]), bool(current["is_day"])
        )
        temperature = round(float(current["temperature_2m"]))
        result = {
            "text": f"{icon}  {temperature}°",
            "tooltip": f"{LOCATION} · {description} · {temperature}°C",
            "class": description.lower().replace(" ", "-"),
            "alt": description,
        }
    except (URLError, TimeoutError, KeyError, TypeError, ValueError, json.JSONDecodeError):
        result = {
            "text": "",
            "tooltip": f"{LOCATION} · Weather unavailable",
            "class": "unavailable",
            "alt": "unavailable",
        }
    print(json.dumps(result, ensure_ascii=False))


if __name__ == "__main__":
    main()
