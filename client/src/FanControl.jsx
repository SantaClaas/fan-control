import { createEffect, createSignal, createResource } from "solid-js";

async function getCurrentSetPoint() {
  const response = await fetch("/api/fan/2/setpoint/current");

  if (!response.ok) {
    return { error: "Failed to fetch setpoint" };
  }

  const data = await response.json();
  return data;
}

/**
 * A slider range input based on https://codepen.io/bgebelein/pen/wvYeapy
 * @returns {import("solid-js").JSX.Element}
 */
function FanControl() {
  /**
   * @type {import("solid-js").Signal<number | undefined>}
   */
  const [setPoint, setSetPoint] = createSignal();
  const [setPointResponse] = createResource(getCurrentSetPoint);
  /**
   * @type {HTMLInputElement | undefined}
   */
  let input;
  createEffect(() => {
    let value = setPoint();
    if (value && input) input.value = value.toString();
  });

  const valueLabel = () => {
    if (setPointResponse.loading) return "Loading";
    if (setPointResponse.error) return "Error loading";
    return setPoint();
  };

  return (
    <>
      <label for="speed" class="block">
        Speed (RPM)
      </label>{" "}
      <input
        id="speed"
        type="range"
        class="inline-block align-middle w-3/4"
        min="0"
        max="100"
        disabled={setPointResponse.loading || setPointResponse.error}
        onInput={(event) => setSetPoint(Number(event.target.value))}
        ref={input}
      />
      <span class="align-middle min-w-[4ch] mx-4">{valueLabel()}</span>
    </>
  );
}

export default FanControl;
