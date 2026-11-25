// the clips play only while they're on screen; clips and screenshots open
// bigger in a floating viewer (clips in their full-size version)
const still = matchMedia("(prefers-reduced-motion: reduce)").matches;
const clips = document.querySelectorAll(".clip video");

if (!still) {
  const seen = new IntersectionObserver(
    (entries) => {
      for (const { target, isIntersecting } of entries) {
        if (isIntersecting) target.play().catch(() => {});
        else target.pause();
      }
    },
    { threshold: 0.4 },
  );
  clips.forEach((video) => seen.observe(video));
}

const viewer = document.querySelector(".viewer");
const image = viewer.querySelector("img");
const video = viewer.querySelector("video");
const caption = viewer.querySelector("p");

function open(label, { src, clip }) {
  image.hidden = !!clip;
  video.hidden = !clip;
  if (clip) {
    video.src = clip;
    if (!still) video.play().catch(() => {});
  } else {
    image.src = src;
    image.alt = label;
  }
  caption.textContent = label;
  viewer.showModal();
}

document.querySelectorAll(".shot").forEach((button) =>
  button.addEventListener("click", () => open(button.dataset.label, { src: button.dataset.full })),
);

document.querySelectorAll(".clip button").forEach((button) =>
  button.addEventListener("click", () => open(button.dataset.label, { clip: button.dataset.video })),
);

viewer.addEventListener("click", (event) => {
  if (event.target === viewer) viewer.close();
});

viewer.addEventListener("close", () => {
  video.pause();
  video.removeAttribute("src");
  video.load();
});
