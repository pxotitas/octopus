// each [data-reveal] part rises in once, when it comes into view; the
// children of a [data-stagger] come in one after another, and the chips at
// the top too
{
  document.querySelectorAll(".jump .chip").forEach((chip, i) => chip.style.setProperty("--i", i));
  document.querySelectorAll("[data-stagger]").forEach((group) => {
    [...group.children].forEach((child, i) => child.style.setProperty("--i", i));
  });

  // once in, the delay goes, so hovering answers right away
  const settle = (part) => {
    const groups = part.matches("[data-stagger]") ? [part] : [...part.querySelectorAll("[data-stagger]")];
    for (const group of groups) {
      const wait = group.children.length * 70 + 800;
      setTimeout(() => [...group.children].forEach((child) => child.style.setProperty("--i", 0)), wait);
    }
  };

  const parts = document.querySelectorAll("[data-reveal]");
  if (matchMedia("(prefers-reduced-motion: reduce)").matches || !("IntersectionObserver" in window)) {
    parts.forEach((part) => part.classList.add("on"));
  } else {
    const seen = new IntersectionObserver(
      (entries) => {
        for (const { target, isIntersecting } of entries) {
          if (!isIntersecting) continue;
          target.classList.add("on");
          settle(target);
          seen.unobserve(target);
        }
      },
      { rootMargin: "0px 0px -12% 0px" },
    );
    parts.forEach((part) => seen.observe(part));
  }
}
