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
      "Server respoded but could not get setpoint. See network tab for details.",
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

function IconFanOff() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      height="24"
      viewBox="0 -960 960 960"
      width="24"
      class="mx-auto"
      fill="currentColor"
    >
      <path d="M880-424q0 51-32 77.5T777-320q-6 0-12-.5t-12-2.5L459-618q8-42 31-78t59-60q6-4 8.5-10.5T560-781q0-8-6.5-13.5T536-800q-38 0-86 16t-50 65q0 16 5 31t13 30l-94-94q13-54 66.5-91T536-880q51 0 77.5 30.5T640-781q0 26-11.5 51T593-689q-22 14-35.5 36T539-606l12 6q6 3 11 7l92-34q17-6 32.5-9.5T719-640q81 0 121 67t40 149ZM819-28 637-211q-13 54-66.5 92.5T424-80q-51 0-77.5-30.5T320-180q0-26 11.5-50.5T367-271q22-14 35.5-36t18.5-47l-12-6q-6-3-11-7l-92 33q-17 6-33 10t-33 4q-63 0-111.5-55T80-536q0-51 30.5-77.5T179-640q8 0 16.5 1t16.5 3L27-820l57-57L876-85l-57 57Zm-42-372q9 0 16-5t7-19q0-38-16-86.5T719-560q-9 0-17 1.5t-15 4.5l-75 28q2 6 3.5 12.5T618-501q42 8 78 30.5t60 59.5q3 5 9 8t12 3Zm-537 0q10 0 18.5-2.5T273-407l75-27q-2-6-3.5-12.5T342-459q-42-8-76.5-29.5T212-538q-9-14-17-18t-16-4q-9 0-14 6t-5 18q0 54 20.5 95t59.5 41Zm184 240q62 0 100-24.5t36-63.5q0-10-4-26t-14-32l-37-37q-11 42-34 78t-60 61q-5 3-8 10t-3 14q2 9 7.5 14.5T424-160Zm194-341Zm-276 42Zm163 116Zm-46-275Z" />
    </svg>
  );
}

function IconFanNight() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      height="24"
      viewBox="0 -960 960 960"
      width="24"
      class="mx-auto"
      fill="currentColor"
    >
      <path d="M480-120q-150 0-255-105T120-480q0-150 105-255t255-105q14 0 27.5 1t26.5 3q-41 29-65.5 75.5T444-660q0 90 63 153t153 63q55 0 101-24.5t75-65.5q2 13 3 26.5t1 27.5q0 150-105 255T480-120Zm0-80q88 0 158-48.5T740-375q-20 5-40 8t-40 3q-123 0-209.5-86.5T364-660q0-20 3-40t8-40q-78 32-126.5 102T200-480q0 116 82 198t198 82Zm-10-270Z" />
    </svg>
  );
}

function IconFanDay() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      height="24"
      viewBox="0 -960 960 960"
      width="24"
      class="mx-auto"
      fill="currentColor"
    >
      <path d="M480-360q50 0 85-35t35-85q0-50-35-85t-85-35q-50 0-85 35t-35 85q0 50 35 85t85 35Zm0 80q-83 0-141.5-58.5T280-480q0-83 58.5-141.5T480-680q83 0 141.5 58.5T680-480q0 83-58.5 141.5T480-280ZM200-440H40v-80h160v80Zm720 0H760v-80h160v80ZM440-760v-160h80v160h-80Zm0 720v-160h80v160h-80ZM256-650l-101-97 57-59 96 100-52 56Zm492 496-97-101 53-55 101 97-57 59Zm-98-550 97-101 59 57-100 96-56-52ZM154-212l101-97 55 53-97 101-59-57Zm326-268Z" />
    </svg>
  );
}

function IconFanParty() {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      height="24"
      viewBox="0 -960 960 960"
      width="24"
      class="mx-auto"
      fill="currentColor"
    >
      <path d="M280-80v-366q-51-14-85.5-56T160-600v-280h80v280h40v-280h80v280h40v-280h80v280q0 56-34.5 98T360-446v366h-80Zm400 0v-320H560v-280q0-83 58.5-141.5T760-880v800h-80Z" />
    </svg>
  );
}
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

  // return (
  //   <fieldset disabled={isDisabled()} class="h-12 rounded-xl bg-cyan-800">
  //     <legend class="sr-only">Speed</legend>
  //     <label>
  //       <input
  //         type="radio"
  //         name="speed"
  //         class="appearance-none"
  //         value={OFF.toString()}
  //         onChange={handleChange}
  //         checked={value() === OFF}
  //       />
  //       <IconFanOff />
  //       <span>Off</span>
  //     </label>
  //     <label>
  //       <input
  //         type="radio"
  //         name="speed"
  //         value={NIGHT.toString()}
  //         onChange={handleChange}
  //         checked={OFF < value() && value() <= NIGHT}
  //       />
  //       Night
  //     </label>
  //     <label>
  //       <input
  //         type="radio"
  //         name="speed"
  //         value={DAY.toString()}
  //         onChange={handleChange}
  //         checked={NIGHT < value() && value() <= DAY}
  //       />
  //       Day
  //     </label>
  //     <label>
  //       <input
  //         type="radio"
  //         name="speed"
  //         value={PARTY.toString()}
  //         onChange={handleChange}
  //         checked={DAY < value() && value() <= PARTY}
  //       />
  //       High
  //     </label>
  //   </fieldset>
  // );

  /**
   * @param {{label: string, children: import("solid-js").JSX.Element}} param0
   * @returns {import("solid-js").JSX.Element}
   */
  function Option({ label, children }) {
    return (
      <label class="block p-2 text-center ring-1 has-[:checked]:bg-teal-900 has-[:checked]:text-teal-50 has-[:checked]:ring-teal-500">
        <input type="radio" name="speed" class="sr-only" />
        {children}
        <span>{label}</span>
      </label>
    );
  }

  return (
    <fieldset class="grid grid-flow-col rounded-xl bg-cyan-800 text-slate-100">
      <legend class="sr-only">Speed</legend>{" "}
      <label class="block rounded-l-xl rounded-r-sm p-2 text-center has-[:checked]:bg-teal-900 has-[:checked]:text-teal-50 has-[:checked]:ring-1 has-[:checked]:ring-teal-500">
        <input type="radio" name="speed" class="sr-only" />
        <IconFanOff />
        <span>Off</span>
      </label>
      <label class="block rounded-sm p-2 text-center has-[:checked]:bg-teal-900 has-[:checked]:text-teal-50 has-[:checked]:ring-1 has-[:checked]:ring-teal-500">
        <input type="radio" name="speed" class="sr-only" />
        <IconFanNight />
        <span>Night</span>
      </label>
      <label class="block rounded-sm p-2 text-center has-[:checked]:bg-teal-900 has-[:checked]:text-teal-50 has-[:checked]:ring-1 has-[:checked]:ring-teal-500">
        <input type="radio" name="speed" class="sr-only" />
        <IconFanDay />
        <span>Day</span>
      </label>
      <label class="block rounded-sm rounded-r-xl p-2 text-center has-[:checked]:bg-teal-900 has-[:checked]:text-teal-50 has-[:checked]:ring-1 has-[:checked]:ring-teal-500">
        <input type="radio" name="speed" class="sr-only" />
        <IconFanParty />
        <span>Party</span>
      </label>
    </fieldset>
  );
}

/**
 * A slider range input based on https://codepen.io/bgebelein/pen/wvYeapy
 * @returns {import("solid-js").JSX.Element}
 */
function FanControl() {
  const isDemo = new URL(window.location.href).searchParams.has("demo");
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
      { signal: controller.signal },
    );

    eventSource.addEventListener(
      "error",
      (event) => {
        console.error("EventSource error:", event);
      },
      { signal: controller.signal },
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
        class="inline-block w-3/4 align-middle"
        min="0"
        max={MAX_SET_POINT}
        value={setPoint() || 0}
        disabled={isDisabled()}
        onInput={(event) => setSetPoint(Number(event.target.value))}
        onChange={handleChange}
        ref={input}
      />
      <span class="mx-4 min-w-[4ch] align-middle">{valueLabel()}</span>
      <ModeFanControl setPoint={setPoint} setSetPoint={setSetPoint} />
    </>
  );
}

export default FanControl;
