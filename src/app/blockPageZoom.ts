// Stops the webview from zooming the whole app with Control and the wheel
// (or a pinch, which arrives as a wheel with Control). The canvas zooms
// itself. The listener is not passive, so preventDefault works.
// Returns a function that removes it.
export function blockPageZoom(): () => void {
  const block = (event: WheelEvent) => {
    if (event.ctrlKey) event.preventDefault();
  };
  window.addEventListener("wheel", block, { passive: false });
  return () => window.removeEventListener("wheel", block);
}
