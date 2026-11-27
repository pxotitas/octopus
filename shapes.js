// the clock's two shapes, the same polar wave the companion draws; they
// click round and change on every key, like the lock screen.
{
  const kinds = [[4, 0.16], [8, 0.11], [4, 0.07], [6, 0.085], [7, 0.07], [9, 0.055], [12, 0.035], [5, 0.09], [8, 0.04]];

  function outline([lobes, depth]) {
    const points = [];
    for (let i = 0; i < 360; i++) {
      const t = (2 * Math.PI * i) / 360;
      const r = (50 * (1 + depth * Math.cos(lobes * t))) / (1 + depth);
      points.push(`${(50 + r * Math.cos(t)).toFixed(2)},${(50 + r * Math.sin(t)).toFixed(2)}`);
    }
    return `M${points.join(" L")}Z`;
  }

  const shapes = [...document.querySelectorAll(".shape")].map((el, i) => {
    const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.setAttribute("viewBox", "0 0 100 100");
    const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
    svg.append(path);
    el.prepend(svg);
    const state = { el, svg, path, kind: i === 0 ? 5 : 0, angle: 0 };
    path.setAttribute("d", outline(kinds[state.kind]));
    return state;
  });

  function adapt() {
    const taken = [];
    for (const s of shapes) {
      let next;
      do next = Math.floor(Math.random() * kinds.length);
      while (next === s.kind || taken.includes(next));
      taken.push(next);
      s.kind = next;
      s.path.setAttribute("d", outline(kinds[next]));
      s.angle += (Math.random() < 0.5 ? -1 : 1) * (1 + Math.floor(Math.random() * 3)) * 22.5;
      s.svg.style.transform = `rotate(${s.angle}deg)`;
    }
  }

  function tick() {
    const now = new Date();
    const [h, m] = document.querySelectorAll(".shape span");
    h.textContent = String(now.getHours()).padStart(2, "0");
    m.textContent = String(now.getMinutes()).padStart(2, "0");
  }

  tick();
  setInterval(tick, 10000);
  document.addEventListener("keydown", adapt);
  document.querySelector(".shapes").addEventListener("click", adapt);
}
