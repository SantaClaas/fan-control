import {
  createEffect,
  createSignal,
  createResource,
  onCleanup,
} from "solid-js";

const MAX_SET_POINT = 64_000;
async function getSetPoint() {
  //TODO error handling
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

  //TODO error handling
  const response = await fetch("/api/fan/2/setpoint", {
    method: "PATCH",
    headers: {
      "Content-Type": "application/json",
    },
    body: setPoint.toString(),
  });
}

const OFF = 0;
const NIGHT = MAX_SET_POINT * 0.1;
const DAY = MAX_SET_POINT * 0.25;
const PARTY = MAX_SET_POINT * 0.5;

/**
 *
 * @param {{setPoint: import("solid-js").Accessor<number | undefined>, setSetPoint: import("solid-js").Setter<number | undefined>}} param0
 * @returns
 */
function ModeFanControl({ setPoint, setSetPoint }) {
  /**
   * @param {{target: HTMLInputElement}} event
   */
  function handleChange(event) {
    console.debug("Setting setpoint", event.target.value);

    const value = Number(event.target.value);
    if (Number.isNaN(value)) return;
    setSetPoint(value);
  }

  const isDisabled = () => setPoint() === undefined;

  const value = () => setPoint() || 0;

  return (
    <fieldset disabled={isDisabled()}>
      <legend>Speed</legend>
      <label>
        <input
          type="radio"
          name="speed"
          value={OFF.toString()}
          onChange={handleChange}
          checked={value() === OFF}
        />
        Off
      </label>
      <label>
        <input
          type="radio"
          name="speed"
          value={NIGHT.toString()}
          onChange={handleChange}
          checked={OFF < value() && value() <= NIGHT}
        />
        Night
      </label>
      <label>
        <input
          type="radio"
          name="speed"
          value={DAY.toString()}
          onChange={handleChange}
          checked={NIGHT < value() && value() <= DAY}
        />
        Day
      </label>
      <label>
        <input
          type="radio"
          name="speed"
          value={PARTY.toString()}
          onChange={handleChange}
          checked={DAY < value() && value() <= PARTY}
        />
        High
      </label>
    </fieldset>
  );
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

  createEffect(() => {
    if (setPointResponse.loading || setPointResponse.error) return;

    setSetPoint(setPointResponse());
  });

  /**
   * @type {HTMLInputElement | undefined}
   */
  let input;
  createEffect(() => {
    let value = setPoint();
    if (value === undefined || Number.isNaN(value)) return;

    // Update server
    updateSetPoint(value);

    // Reflect change in input
    if (value && input) input.value = value.toString();
  });

  const valueLabel = () => {
    if (setPointResponse.loading) return "Loading";
    if (setPointResponse.error) return "Error";
    const value = setPoint();
    console.debug("Value", value);
    if (value === undefined) return "Not loaded";
    return (value / MAX_SET_POINT).toLocaleString(undefined, {
      style: "percent",
      maximumFractionDigits: 0,
    });
  };

  const isDisabled = () => setPointResponse.loading || setPointResponse.error;

  /**
   * @param {{target: HTMLInputElement}} event
   */
  function handleChange(event) {
    if (isDisabled()) return;

    const value = Number(event.target.value);
    if (Number.isNaN(value)) return;
    setSetPoint(value);
  }

  createEffect(() => {
    console.log("Creating event source");

    const controller = new AbortController();
    const eventSource = new EventSource("/api/sse");
    eventSource.addEventListener(
      "message",
      (event) => {
        if (event.data === "None") return;

        const value = Number(event.data);

        console.debug("Received set point update", value);
        if (value === setPoint()) return;

        setSetPoint(value);
      },
      { signal: controller.signal }
    );

    eventSource.addEventListener(
      "error",
      (event) => {
        console.error("EventSource error:", event);
      },
      { signal: controller.signal }
    );

    onCleanup(() => {
      console.debug("Cleaning up event source");
      controller.abort();
      eventSource.close();
    });
  });

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
        disabled={isDisabled()}
        onInput={(event) => setSetPoint(Number(event.target.value))}
        onChange={handleChange}
        ref={input}
      />
      <span class="align-middle min-w-[4ch] mx-4">{valueLabel()}</span>
      <ModeFanControl setPoint={setPoint} setSetPoint={setSetPoint} />
    </>
  );
}

export default FanControl;
