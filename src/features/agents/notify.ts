// A system notification, for a new agent that waits while the window is in
// the background. It uses the notification of the web view (no new
// dependency); where the system does not support it, nothing happens and the
// prompt is still there when the person comes back.

export function notifyApproval(title: string, body: string) {
  try {
    if (typeof Notification === "undefined" || document.hasFocus()) return;
    const show = () => new Notification(title, { body });
    if (Notification.permission === "granted") show();
    else if (Notification.permission === "default") {
      void Notification.requestPermission().then((answer) => {
        if (answer === "granted") show();
      });
    }
  } catch {
    // A notification is only a help.
  }
}
