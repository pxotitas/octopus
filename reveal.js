// each [data-reveal] part rises in once, when it comes into view; the
// chips at the top come in one after another
{
  document.querySelectorAll(".jump .chip").forEach((chip, i) => chip.style.setProperty("--i", i));

  const parts = document.querySelectorAll("[data-reveal]");
  if (matchMedia("(prefers-reduced-motion: reduce)").matches || !("IntersectionObserver" in window)) {
    parts.forEach((part) => part.classList.add("on"));
  } else {
    const seen = new IntersectionObserver(
      (entries) => {
        for (const { target, isIntersecting } of entries) {
          if (!isIntersecting) continue;
          target.classList.add("on");
          seen.unobserve(target);
        }
      },
      { rootMargin: "0px 0px -15% 0px" },
    );
    parts.forEach((part) => seen.observe(part));
  }
}
