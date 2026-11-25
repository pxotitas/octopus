// M3 expressive shapes, the same polar wave the shell draws: every
// `.shape-badge` gets its outline and morphs into `data-hover` under the
// pointer.
const KINDS = {
  circle: [0, 0, "wave"],
  clover4: [4, 0.16, "wave"],
  clover8: [8, 0.11, "wave"],
  cookie4: [4, 0.07, "wave"],
  cookie6: [6, 0.085, "wave"],
  cookie7: [7, 0.07, "wave"],
  cookie9: [9, 0.055, "wave"],
  cookie12: [12, 0.035, "wave"],
  pentagon: [5, 0.09, "wave"],
  sunny: [8, 0.05, "soft"],
  verysunny: [8, 0.11, "spike"],
  flower: [8, 0.2, "puff"],
  puffy: [10, 0.06, "puff"],
  burst: [12, 0.17, "spike"],
  softburst: [10, 0.1, "soft"],
};

const STEPS = 180;
const still = matchMedia("(prefers-reduced-motion: reduce)").matches;

function lobe(profile, x) {
  const wave = Math.cos(x);
  const spike = 1 - (2 * Math.acos(Math.max(-1, Math.min(1, wave)))) / Math.PI;
  if (profile === "puff") return 2 * Math.abs(Math.cos(x / 2)) - 1;
  if (profile === "spike") return spike;
  if (profile === "soft") return (wave + spike) / 2;
  return wave;
}

function radii(kind) {
  const [lobes, depth, profile] = KINDS[kind] || KINDS.circle;
  return Array.from({ length: STEPS }, (_, i) => {
    const t = (i / STEPS) * 2 * Math.PI;
    return (1 + depth * lobe(profile, lobes * t)) / (1 + depth);
  });
}

function path(rs) {
  return (
    "M" +
    rs
      .map((r, i) => {
        const t = (i / STEPS) * 2 * Math.PI - Math.PI / 2;
        return `${(50 + 50 * r * Math.cos(t)).toFixed(2)},${(50 + 50 * r * Math.sin(t)).toFixed(2)}`;
      })
      .join(" L") +
    "Z"
  );
}

const ease = (t) => 1 - Math.pow(1 - t, 3);

for (const badge of document.querySelectorAll(".shape-badge")) {
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("viewBox", "0 0 100 100");
  svg.setAttribute("aria-hidden", "true");
  const outline = document.createElementNS("http://www.w3.org/2000/svg", "path");
  svg.append(outline);
  badge.prepend(svg);

  const rest = radii(badge.dataset.shape);
  const hover = radii(badge.dataset.hover || badge.dataset.shape);
  let current = rest.slice();
  outline.setAttribute("d", path(current));
  if (still || !badge.dataset.hover) continue;

  let frame = 0;
  const morph = (target) => {
    cancelAnimationFrame(frame);
    const from = current.slice();
    const start = performance.now();
    const step = (now) => {
      const p = ease(Math.min(1, (now - start) / 380));
      current = from.map((r, i) => r + (target[i] - r) * p);
      outline.setAttribute("d", path(current));
      if (p < 1) frame = requestAnimationFrame(step);
    };
    frame = requestAnimationFrame(step);
  };
  const host = badge.closest("a, button, .feature, .insight") || badge;
  host.addEventListener("pointerenter", () => morph(hover));
  host.addEventListener("pointerleave", () => morph(rest));
}
