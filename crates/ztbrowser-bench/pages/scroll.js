(function () {
  const SPEED = /*SPEED_PX_PER_SECOND*/0, DURATION = /*DURATION_MS*/0, SETTLE = /*SETTLE_MS*/0;
  addEventListener("load", () => {
    fetch("/*LOADED_URL*/", { mode: "no-cors" });
    setTimeout(run, SETTLE);
  });
  function run() {
    const gaps = [];
    let last = performance.now(), start = last, y = scrollY, direction = 1;
    const maximum = () => document.documentElement.scrollHeight - innerHeight;
    function frame(now) {
      gaps.push(now - last);
      last = now;
      y += direction * SPEED * gaps[gaps.length - 1] / 1000;
      if (y >= maximum()) { y = maximum(); direction = -1; } else if (y <= 0) { y = 0; direction = 1; }
      scrollTo(0, y);
      if (now - start < DURATION) { requestAnimationFrame(frame); return; }
      gaps.shift();
      fetch("/*RESULT_URL*/", { method: "POST", body: JSON.stringify(gaps) });
    }
    requestAnimationFrame(frame);
  }
})();
