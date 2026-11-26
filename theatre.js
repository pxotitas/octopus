// "some features": the curtain, the reveals, the three ways to watch, and
// the small live demos (the shape cast, the desktop, the bun clock, the lock).
const theatre = document.querySelector(".theatre");
if (theatre) {
  const still = matchMedia("(prefers-reduced-motion: reduce)").matches;
  const store = {
    get: () => { try { return localStorage.getItem("octopus-mode"); } catch { return null; } },
    set: (v) => { try { localStorage.setItem("octopus-mode", v); } catch {} },
  };

  // modes
  const buttons = theatre.querySelectorAll(".modes [data-mode]");
  const setMode = (mode) => {
    theatre.dataset.mode = mode;
    buttons.forEach((b) => b.setAttribute("aria-checked", String(b.dataset.mode === mode)));
  };
  setMode(store.get() || (still ? "script" : "theatre"));
  buttons.forEach((b) =>
    b.addEventListener("click", () => {
      const swap = () => { setMode(b.dataset.mode); store.set(b.dataset.mode); };
      if (document.startViewTransition && !still) document.startViewTransition(swap);
      else swap();
    }),
  );

  // the curtain opens once the page is up
  requestAnimationFrame(() => setTimeout(() => theatre.classList.add("open"), still ? 0 : 250));

  // reveals, staggered inside [data-stagger]
  const seen = new IntersectionObserver(
    (entries) => entries.forEach((e) => { if (e.isIntersecting) { e.target.classList.add("on"); seen.unobserve(e.target); } }),
    { rootMargin: "0px 0px -12% 0px" },
  );
  const watch = () => theatre.querySelectorAll("[data-reveal]:not(.on)").forEach((el) => seen.observe(el));

  // the chiaroscuro spotlight follows the pointer
  theatre.addEventListener("pointermove", (e) => {
    theatre.style.setProperty("--x", `${e.clientX}px`);
    theatre.style.setProperty("--y", `${e.clientY}px`);
  });

  // parallax for [data-depth]
  const deep = theatre.querySelectorAll("[data-depth]");
  let ticking = false;
  const drift = () => {
    ticking = false;
    const mid = innerHeight / 2;
    deep.forEach((el) => {
      const r = el.getBoundingClientRect();
      el.style.setProperty("--shift", `${((r.top + r.height / 2 - mid) * -Number(el.dataset.depth)).toFixed(1)}px`);
    });
    const done = Math.min(1, scrollY / Math.max(1, document.body.scrollHeight - innerHeight));
    theatre.style.setProperty("--progress", done.toFixed(3));
  };
  if (!still) addEventListener("scroll", () => { if (!ticking) { ticking = true; requestAnimationFrame(drift); } }, { passive: true });

  // act ii: the whole cast of shapes; each turns into the next one
  const cast = theatre.querySelector(".shape-cast");
  const kinds = ["circle", "clover4", "clover8", "cookie4", "cookie6", "cookie7", "cookie9", "cookie12",
    "pentagon", "sunny", "verysunny", "flower", "puffy", "burst", "softburst"];
  const tones = ["", "tone-b", "tone-c"];
  kinds.forEach((kind, i) => {
    const item = document.createElement("button");
    item.type = "button";
    item.className = "cast";
    item.dataset.reveal = "rise";
    item.style.setProperty("--i", i % 8);
    item.innerHTML = `<span class="shape-badge big ${tones[i % 3]}" data-shape="${kind}" data-hover="${kinds[(i + 5) % kinds.length]}"></span><span>${kind}</span>`;
    cast.append(item);
  });

  // act iii: a desktop whose widgets move and resize like the real ones
  const desk = theatre.querySelector(".desk");
  const grid = theatre.querySelector(".contrib");
  let seed = 7;
  const random = () => ((seed = (seed * 16807) % 2147483647) / 2147483647);
  let total = 0;
  for (let i = 0; i < 7 * 40; i++) {
    const week = Math.floor(i / 7);
    const busy = random() < 0.25 + week / 70;
    const level = busy ? 1 + Math.floor(random() * 4) : 0;
    total += level * 2;
    const cell = document.createElement("i");
    cell.dataset.level = level;
    cell.style.setProperty("--d", `${week * 18}ms`);
    grid.append(cell);
  }
  theatre.querySelector(".desk-total").textContent = `${total}`;
  // as many weeks as fit, the squares as tall as a seventh of the widget
  new ResizeObserver(([entry]) => {
    grid.style.setProperty("--pitch", `${Math.max(6, entry.contentRect.height / 7).toFixed(2)}px`);
  }).observe(grid);
  desk.querySelectorAll(".desk-widget").forEach((widget) => {
    widget.addEventListener("pointerdown", (e) => {
      const box = desk.getBoundingClientRect();
      const resize = e.target.classList.contains("grip");
      const start = { x: e.clientX, y: e.clientY, l: widget.offsetLeft, t: widget.offsetTop, w: widget.offsetWidth, h: widget.offsetHeight };
      widget.setPointerCapture(e.pointerId);
      widget.classList.add("held");
      desk.querySelectorAll(".desk-widget").forEach((w) => (w.style.zIndex = w === widget ? 2 : 1));
      const move = (m) => {
        const dx = m.clientX - start.x, dy = m.clientY - start.y;
        if (resize) {
          widget.style.setProperty("--w", `${Math.max(120, Math.min(box.width - start.l, start.w + dx))}px`);
          widget.style.setProperty("--h", `${Math.max(90, Math.min(box.height - start.t, start.h + dy))}px`);
        } else {
          widget.style.setProperty("--x", `${Math.max(0, Math.min(box.width - start.w, start.l + dx))}px`);
          widget.style.setProperty("--y", `${Math.max(0, Math.min(box.height - start.h, start.t + dy))}px`);
        }
      };
      const up = () => {
        widget.classList.remove("held");
        widget.removeEventListener("pointermove", move);
        widget.removeEventListener("pointerup", up);
        widget.removeEventListener("pointercancel", up);
      };
      widget.addEventListener("pointermove", move);
      widget.addEventListener("pointerup", up);
      widget.addEventListener("pointercancel", up);
      e.preventDefault();
    });
  });

  // act iv: the bun clock, on the reader's time
  const hh = theatre.querySelector(".bun .hh");
  const mm = theatre.querySelector(".bun .mm");
  const tick = () => {
    const now = new Date();
    hh.textContent = String(now.getHours()).padStart(2, "0");
    mm.textContent = String(now.getMinutes()).padStart(2, "0");
  };
  tick();
  setInterval(tick, 5000);

  // act v: the lock's shapes change at every key
  const lock = theatre.querySelector(".lock-shapes");
  const input = theatre.querySelector(".lock-demo input");
  const pick = () => kinds[Math.floor(Math.random() * kinds.length)];
  for (let i = 0; i < 6; i++) {
    const badge = document.createElement("span");
    badge.className = `shape-badge big ${tones[i % 3]}`;
    badge.dataset.shape = pick();
    lock.append(badge);
  }
  input.addEventListener("input", () => {
    lock.querySelectorAll(".shape-badge").forEach((badge, i) => {
      if (badge.morphTo) setTimeout(() => badge.morphTo(pick()), i * 40);
    });
    lock.style.setProperty("--turn", `${input.value.length * 30}deg`);
  });

  watch();
  drift();
  // shapes made here need m3.js, which runs after this file
  window.addEventListener("DOMContentLoaded", watch);
}
