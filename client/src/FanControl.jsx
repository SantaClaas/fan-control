import { createEffect, createSignal, createResource } from "solid-js";

const MAX_SET_POINT = 64_000;
async function getSetPoint() {
  const response = await fetch("/api/fan/2/setpoint");

  if (!response.ok)
    throw new Error(
      "Server respoded but could not get setpoint. See network tab for details."
    );

  const data = await response.json();
  return data;
}

/**
 * @param {Number} setPoint
 */
async function updateSetPoint(setPoint) {
  if (setPoint < 0 || setPoint > MAX_SET_POINT) {
    throw new Error("Setpoint out of range");
  }

  const response = await fetch("/api/fan/2/setpoint", {
    method: "PATCH",
    headers: {
      "Content-Type": "application/json",
    },
    body: setPoint.toString(),
  });
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
  const [setPointResponse] = createResource(getSetPoint);
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
    if (setPointResponse.error) return "Error";
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
        max={MAX_SET_POINT}
        value={setPoint() || 0}
        disabled={setPointResponse.loading || setPointResponse.error}
        onInput={(event) => setSetPoint(Number(event.target.value))}
        onChange={(event) => updateSetPoint(Number(event.target.value))}
        ref={input}
      />
      <span class="align-middle min-w-[4ch] mx-4">{valueLabel()}</span>
    </>
  );
}

export default FanControl;
