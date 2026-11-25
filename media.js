// the clips play only while they're on screen; screenshots open full size
const still = matchMedia("(prefers-reduced-motion: reduce)").matches;
const clips = document.querySelectorAll(".clip video");

if (still) {
  clips.forEach((video) => (video.controls = true));
} else {
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
const full = viewer.querySelector("img");
const caption = viewer.querySelector("p");

document.querySelectorAll(".shot").forEach((button) =>
  button.addEventListener("click", () => {
    full.src = button.dataset.full;
    full.alt = caption.textContent = button.dataset.label;
    viewer.showModal();
  }),
);

viewer.addEventListener("click", (event) => {
  if (event.target === viewer) viewer.close();
});
