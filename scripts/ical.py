#!/usr/bin/env python3
"""Events from iCal addresses, for the dash's calendar and the agenda widget.

    ical.py URL [URL...]

Takes any .ics address: Google Calendar's "secret address in iCal format"
(Settings → your calendar → Integrate calendar), Outlook's, Proton's,
Nextcloud's... No account or key. Repeating events are expanded from five
weeks back to ten weeks ahead. Each address is kept in ~/.cache/octopus so
the calendar still shows offline. Prints JSON: {"events": [...], "errors": [...]}.
"""

import datetime as dt
import hashlib
import json
import os
import pathlib
import re
import sys
import urllib.request
from zoneinfo import ZoneInfo

from dateutil import rrule

CACHE = pathlib.Path(os.environ.get("XDG_CACHE_HOME", pathlib.Path.home() / ".cache")) / "octopus" / "calendar"
LOCAL = dt.datetime.now().astimezone().tzinfo
TODAY = dt.date.today()
FROM = dt.datetime.combine(TODAY - dt.timedelta(days=35), dt.time(), LOCAL)
UNTIL = dt.datetime.combine(TODAY + dt.timedelta(days=70), dt.time(), LOCAL)


def fetch(url):
    url = re.sub(r"^webcal://", "https://", url.strip())
    CACHE.mkdir(parents=True, exist_ok=True)
    cached = CACHE / (hashlib.sha1(url.encode()).hexdigest()[:16] + ".ics")
    try:
        request = urllib.request.Request(url, headers={"User-Agent": "octopus-shell"})
        with urllib.request.urlopen(request, timeout=20) as answer:
            text = answer.read().decode("utf-8", "replace")
        if "BEGIN:VCALENDAR" not in text:
            raise ValueError("not an iCal file")
        cached.write_text(text)
        return text, None
    except Exception as err:
        if cached.exists():
            return cached.read_text(), f"{url[:48]}…: offline, showing the last copy"
        return None, f"{url[:48]}…: {err}"


def lines(text):
    """Unfolds the continuation lines and splits each into name, params, value."""
    unfolded = re.sub(r"\r?\n[ \t]", "", text)
    for line in unfolded.splitlines():
        if ":" not in line:
            continue
        head, value = line.split(":", 1)
        name, *params = head.split(";")
        options = {}
        for p in params:
            if "=" in p:
                k, v = p.split("=", 1)
                options[k.upper()] = v.strip('"')
        yield name.upper(), options, value


def unescape(value):
    return value.replace("\\n", " ").replace("\\N", " ").replace("\\,", ",").replace("\\;", ";").replace("\\\\", "\\")


def moment(value, options):
    """A datetime in the local zone, and whether it is a whole day."""
    value = value.strip()
    if options.get("VALUE") == "DATE" or re.fullmatch(r"\d{8}", value):
        day = dt.datetime.strptime(value[:8], "%Y%m%d")
        return day.replace(tzinfo=LOCAL), True
    stamp = dt.datetime.strptime(value[:15], "%Y%m%dT%H%M%S")
    if value.endswith("Z"):
        stamp = stamp.replace(tzinfo=dt.timezone.utc)
    elif "TZID" in options:
        try:
            stamp = stamp.replace(tzinfo=ZoneInfo(options["TZID"]))
        except Exception:
            stamp = stamp.replace(tzinfo=LOCAL)
    else:
        stamp = stamp.replace(tzinfo=LOCAL)
    return stamp, False


def duration(value):
    match = re.fullmatch(r"([+-])?P(?:(\d+)W)?(?:(\d+)D)?(?:T(?:(\d+)H)?(?:(\d+)M)?(?:(\d+)S)?)?", value.strip())
    if not match:
        return dt.timedelta(hours=1)
    sign, w, d, h, m, s = match.groups()
    span = dt.timedelta(weeks=int(w or 0), days=int(d or 0), hours=int(h or 0), minutes=int(m or 0), seconds=int(s or 0))
    return -span if sign == "-" else span


def events(text, index):
    calendar = ""
    found, current = [], None
    for name, options, value in lines(text):
        if name == "X-WR-CALNAME":
            calendar = unescape(value)
        elif name == "BEGIN" and value == "VEVENT":
            current = {"exdates": [], "rrule": None}
        elif name == "END" and value == "VEVENT" and current is not None:
            found.append(current)
            current = None
        elif current is not None:
            if name == "DTSTART":
                current["start"], current["allDay"] = moment(value, options)
            elif name == "DTEND":
                current["end"], _ = moment(value, options)
            elif name == "DURATION":
                current["duration"] = duration(value)
            elif name == "SUMMARY":
                current["title"] = unescape(value)
            elif name == "LOCATION":
                current["location"] = unescape(value)
            elif name == "UID":
                current["uid"] = value
            elif name == "RRULE":
                current["rrule"] = value
            elif name == "EXDATE":
                for part in value.split(","):
                    current["exdates"].append(moment(part, options)[0])
            elif name == "RECURRENCE-ID":
                current["recurrence"] = moment(value, options)[0]
            elif name == "STATUS":
                current["cancelled"] = value.upper() == "CANCELLED"

    # occurrences moved or cancelled one by one replace their slot in the series
    moved = {(e.get("uid"), e["recurrence"]) for e in found if "recurrence" in e and "start" in e}
    out = []
    for e in found:
        if "start" not in e:
            continue
        start = e["start"]
        span = (e["end"] - start) if "end" in e else e.get("duration", dt.timedelta(days=1) if e["allDay"] else dt.timedelta(hours=1))
        if e.get("cancelled"):
            continue
        starts = [start]
        if e["rrule"] and "recurrence" not in e:
            rule = e["rrule"]
            if start.tzinfo is not None:
                # dateutil wants UNTIL in UTC when the start has a zone
                rule = re.sub(r"UNTIL=(\d{8})(;|$)", r"UNTIL=\g<1>T235959Z\2", rule)
            try:
                series = rrule.rrulestr(rule, dtstart=start)
                starts = series.between(FROM - span, UNTIL, inc=True)
            except Exception:
                starts = [start]
            skip = {x.astimezone(dt.timezone.utc) for x in e["exdates"]}
            starts = [s for s in starts if s.astimezone(dt.timezone.utc) not in skip]
            starts = [s for s in starts if (e.get("uid"), s) not in moved]
        for s in starts:
            end = s + span
            if end < FROM or s > UNTIL:
                continue
            local_start, local_end = s.astimezone(LOCAL), end.astimezone(LOCAL)
            out.append({
                "title": e.get("title", "(sem título)"),
                "start": local_start.isoformat(),
                "end": local_end.isoformat(),
                "allDay": e["allDay"],
                "location": e.get("location", ""),
                "calendar": calendar,
                "index": index,
            })
    return out


def main(urls):
    found, errors = [], []
    for index, url in enumerate(urls):
        text, error = fetch(url)
        if error:
            errors.append(error)
        if text:
            try:
                found += events(text, index)
            except Exception as err:
                errors.append(f"{url[:48]}…: {err}")
    found.sort(key=lambda e: (e["start"], not e["allDay"]))
    print(json.dumps({"events": found, "errors": errors}, ensure_ascii=False))


if __name__ == "__main__":
    main([a for a in sys.argv[1:] if a.strip()])
