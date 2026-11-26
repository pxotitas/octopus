// autoplaying videos stop and get controls when the system asks for less motion
{
  if (matchMedia("(prefers-reduced-motion: reduce)").matches) {
    document.querySelectorAll("video[autoplay]").forEach((video) => {
      video.pause();
      video.removeAttribute("autoplay");
      video.controls = true;
    });
  }
}
