// The clock's two shapes use the same polar wave the companion draws.
// Keep them still on the landing page; the clock itself continues updating.
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

  const shapes = [...document.querySelectorAll(".shape")];
  shapes.forEach((el, i) => {
    const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.setAttribute("viewBox", "0 0 100 100");
    svg.setAttribute("aria-hidden", "true");
    const path = document.createElementNS("http://www.w3.org/2000/svg", "path");
    svg.append(path);
    el.prepend(svg);
    path.setAttribute("d", outline(kinds[i === 0 ? 7 : 0]));
  });

  function tick() {
    const now = new Date();
    const [h, m] = document.querySelectorAll(".shape span");
    if (!h || !m) return;
    h.textContent = String(now.getHours()).padStart(2, "0");
    m.textContent = String(now.getMinutes()).padStart(2, "0");
  }

  tick();
  setInterval(tick, 10000);
}
