// The home page's weather question: the same words the shell's `>` line
// takes, showing the screenshot of that answer.
{
  const demo = document.querySelector(".ask");
  if (demo) {
    const input = demo.querySelector("input");
    const picture = demo.querySelector(".ask-shot img");
    const caption = demo.querySelector(".ask-shot figcaption");
    const chips = demo.querySelectorAll(".chip[data-ask]");
    const base = picture.dataset.base;

    const answers = [
      ["weather-tomorrow", ["amanha", "tomorrow", "amanhã"]],
      ["weather-yesterday", ["ontem", "yesterday"]],
      ["weather-weekend", ["fim de semana", "fds", "weekend"]],
      ["weather-week", ["semana", "week", "7 dias", "7 days"]],
      ["weather", ["agora", "now", "hoje", "today", ""]],
    ];

    const fold = (text) =>
      text.toLowerCase().normalize("NFD").replace(/[̀-ͯ]/g, "").replace(/^>\s*/, "").trim();

    function show(text) {
      const words = fold(text);
      const hit = answers.find(([, keys]) => keys.some((k) => words === fold(k) || (k && words.includes(fold(k)))));
      if (!hit) return;
      const [name] = hit;
      if (picture.getAttribute("src").endsWith(`${name}.webp`)) return;
      picture.classList.add("swap");
      setTimeout(() => {
        picture.src = `${base}${name}.webp`;
        picture.classList.remove("swap");
      }, 160);
      caption.textContent = `> ${text.replace(/^>\s*/, "") || caption.dataset.now}`;
      chips.forEach((c) => c.setAttribute("aria-pressed", String(fold(c.dataset.ask) === words)));
    }

    input.addEventListener("input", () => show(input.value));
    chips.forEach((chip) =>
      chip.addEventListener("click", () => {
        input.value = chip.dataset.ask;
        show(chip.dataset.ask);
      }),
    );
  }
}
