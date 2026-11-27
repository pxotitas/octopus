// the home page: numbers that count up as they come into view, and a copy
// button on the install commands
{
  const still = matchMedia("(prefers-reduced-motion: reduce)").matches;

  const count = (el) => {
    const to = Number(el.dataset.count);
    if (still || !to) return;
    const start = performance.now();
    const length = 1100;
    const step = (now) => {
      const t = Math.min(1, (now - start) / length);
      const eased = 1 - Math.pow(1 - t, 4);
      el.textContent = Math.round(to * eased);
      if (t < 1) requestAnimationFrame(step);
    };
    el.textContent = "0";
    requestAnimationFrame(step);
  };

  const numbers = document.querySelectorAll("[data-count]");
  if ("IntersectionObserver" in window) {
    const seen = new IntersectionObserver((entries) => {
      for (const { target, isIntersecting } of entries) {
        if (!isIntersecting) continue;
        count(target);
        seen.unobserve(target);
      }
    }, { threshold: 0.6 });
    numbers.forEach((n) => seen.observe(n));
  }

  const copied = document.documentElement.lang.startsWith("pt") ? "copiado" : "copied";
  const label = document.documentElement.lang.startsWith("pt") ? "copiar" : "copy";
  document.querySelectorAll("pre.copyable").forEach((pre) => {
    const button = document.createElement("button");
    button.className = "copy";
    button.type = "button";
    button.setAttribute("aria-label", label);
    button.innerHTML = '<span class="symbol">content_copy</span>';
    button.addEventListener("click", async () => {
      try {
        await navigator.clipboard.writeText(pre.querySelector("code").textContent);
      } catch {
        return;
      }
      button.classList.add("done");
      button.firstChild.textContent = "check";
      button.setAttribute("aria-label", copied);
      setTimeout(() => {
        button.classList.remove("done");
        button.firstChild.textContent = "content_copy";
        button.setAttribute("aria-label", label);
      }, 1600);
    });
    pre.append(button);
  });
}
