import {
  createEffect,
  createSignal,
  createResource,
  onCleanup,
} from "solid-js";

const MAX_SET_POINT = 64_000;
/**
 *
 * @param {boolean} isDemo
 * @returns {Promise<number | undefined>}
 */
async function getSetPoint(isDemo) {
  if (isDemo) return;
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

  //TODO add swipe over animation when switching modes
  return (
    <fieldset
      disabled={isDisabled()}
      class="grid grid-flow-col rounded-xl bg-cyan-800 text-slate-100"
    >
      <legend class="sr-only">Speed</legend>{" "}
      <label class="block rounded-l-xl rounded-r-sm p-2 text-center has-[:checked]:bg-teal-900 has-[:checked]:text-teal-50 has-[:checked]:ring-1 has-[:checked]:ring-teal-500">
        <input
          type="radio"
          name="speed"
          class="sr-only"
          value={OFF.toString()}
          onChange={handleChange}
          checked={value() === OFF}
        />
        <IconFanOff />
        <span>Off</span>
      </label>
      <label class="block rounded-sm p-2 text-center has-[:checked]:bg-teal-900 has-[:checked]:text-teal-50 has-[:checked]:ring-1 has-[:checked]:ring-teal-500">
        <input
          type="radio"
          name="speed"
          class="sr-only"
          value={NIGHT.toString()}
          onChange={handleChange}
          checked={OFF < value() && value() <= NIGHT}
        />
        <IconFanNight />
        <span>Night</span>
      </label>
      <label class="block rounded-sm p-2 text-center has-[:checked]:bg-teal-900 has-[:checked]:text-teal-50 has-[:checked]:ring-1 has-[:checked]:ring-teal-500">
        <input
          type="radio"
          name="speed"
          class="sr-only"
          value={DAY.toString()}
          onChange={handleChange}
          checked={NIGHT < value() && value() <= DAY}
        />
        <IconFanDay />
        <span>Day</span>
      </label>
      <label class="block rounded-sm rounded-r-xl p-2 text-center has-[:checked]:bg-teal-900 has-[:checked]:text-teal-50 has-[:checked]:ring-1 has-[:checked]:ring-teal-500">
        <input
          type="radio"
          name="speed"
          class="sr-only"
          value={PARTY.toString()}
          onChange={handleChange}
          checked={DAY < value()}
        />
        <IconFanParty />
        <span>Party</span>
      </label>
    </fieldset>
  );
}
/**
 * @param {Function} callback
 * @param {number} delayMilliseconds
 * @returns {(...args: any[]) => void}
 */
function debounce(callback, delayMilliseconds = 250) {
  /**
   * @type {NodeJS.Timeout | undefined}
   */
  let timeoutId;

  return (...args) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => callback(...args), delayMilliseconds);
  };
}

/**
 * @param {Function} callback
 * @param {number} delayMilliseconds
 * @returns {(...args: any[]) => void}
 */
function throttle(callback, delayMilliseconds = 250) {
  let isWaiting = false;
  /** @type {any[] | undefined} */
  let waitingCallArguments;

  function executeLastOrUnblockNext() {
    // If no execution is waiting, reset flag so the next call can be executed
    if (!waitingCallArguments) {
      isWaiting = false;
      return;
    }

    // If there is a call waiting, execute it and block the next calls until the timeout
    callback(...waitingCallArguments);
    waitingCallArguments = undefined;
    isWaiting = true;

    // Allow new calls after delay
    setTimeout(executeLastOrUnblockNext, delayMilliseconds);
  }

  return (...args) => {
    if (isWaiting) {
      // We assume the callback is the same function so we store the latest arguments
      // Set arguments for next call to latest call arguments
      waitingCallArguments = args;
      return;
    }

    // If no call is delayed execute immediately and block the next calls until the timeout
    callback(...args);
    isWaiting = true;
    // Allow new calls after delay
    setTimeout(executeLastOrUnblockNext, delayMilliseconds);
  };
}

/**
 * A slider range input based on https://codepen.io/bgebelein/pen/wvYeapy
 * @returns {import("solid-js").JSX.Element}
 */
function FanControl() {
  const isDemo = true;
  /**
   * @type {import("solid-js").Signal<number | undefined>}
   */
  const [setPoint, setSetPoint] = createSignal();
  const [setPointResponse] = createResource(isDemo, getSetPoint);
  //TODO get maxRpm from server and fan
  const [maxRpm, setMaxRpm] = createSignal(2_000);

  const rpm = () => Math.round(((setPoint() ?? 0) / MAX_SET_POINT) * maxRpm());

  //TODO get and update volume flow from server
  const [volumeFlow, setVolumeFlow] = createSignal(1_000);

  createEffect(() => {
    if (isDemo || setPointResponse.loading || setPointResponse.error) return;

    setSetPoint(setPointResponse());
  });

  /**
   * @type {HTMLInputElement | undefined}
   */
  let input;
  createEffect(() => {
    if (isDemo) return;

    let value = setPoint();
    if (value === undefined || Number.isNaN(value)) return;

    // Update server
    updateSetPoint(value);

    // Reflect change in input
    if (value && input) input.value = value.toString();
  });

  const isDisabled = () => setPointResponse.loading || setPointResponse.error;

  // Handle updates to set point from other clients sent through server-sent events
  //TODO remove uneccesarry effect here
  createEffect(() => {
    if (isDemo) return;
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

  /** @param {{ target: HTMLInputElement; }} event */
  function handleInput(event) {
    // clearTimeout(inputDebounceTimeout);
    if (isDisabled()) return;
    const value = Number(event.target.value);
    if (Number.isNaN(value)) return;

    throttle(setSetPoint, 250)(value);
  }

  return (
    <>
      <main class="grid h-[100dvh] grid-rows-[auto_1fr_auto_auto] gap-4 bg-cyan-950 p-4 text-slate-50">
        {isDemo && (
          <p class="absolute right-0 top-0 -translate-y-1/2 translate-x-1/2 rotate-45 bg-red-600 px-12 pb-1 pt-24 text-2xl">
            Demo
          </p>
        )}
        {/* Need to hide overflow as the width of the viewport increases to the diameter of the rotating square image of
        the fan rotates causing jittery horizontal scroll */}
        <section class="overflow-hidden">
          {/* TODO implement animation winddown curve */}
          <svg
            xmlns="http://www.w3.org/2000/svg"
            height="24"
            viewBox="0 -960 960 960"
            width="24"
            fill="currentColor"
            style={{ "--rpm": rpm() }}
            class="h-full w-full animate-[spin_calc(60s/var(--rpm))_linear_infinite] rounded-full fill-cyan-300/30"
          >
            <path d="M424-80q-51 0-77.5-30.5T320-180q0-26 11.5-50.5T367-271q22-14 35.5-36t18.5-47l-12-6q-6-3-11-7l-92 33q-17 6-33 10t-33 4q-63 0-111.5-55T80-536q0-51 30.5-77.5T179-640q26 0 51 11.5t41 35.5q14 22 36 35.5t47 18.5l6-12q3-6 7-11l-33-92q-6-17-10-33t-4-32q0-64 55-112.5T536-880q51 0 77.5 30.5T640-781q0 26-11.5 51T593-689q-22 14-35.5 36T539-606l12 6q6 3 11 7l92-34q17-6 32.5-9.5T719-640q81 0 121 67t40 149q0 51-32 77.5T777-320q-25 0-48.5-11.5T689-367q-14-22-36-35.5T606-421l-6 12q-3 6-7 11l33 92q6 16 10 30.5t4 30.5q1 65-54 115T424-80Zm56-340q25 0 42.5-17.5T540-480q0-25-17.5-42.5T480-540q-25 0-42.5 17.5T420-480q0 25 17.5 42.5T480-420Z" />
          </svg>
        </section>
        <section class="mx-auto grid h-min w-max grid-cols-[1fr_auto] grid-rows-2 gap-x-1 text-cyan-300/30">
          <p class="col-span-2 grid w-full grid-cols-subgrid items-end">
            <span class="text-end text-5xl font-light">{rpm()}</span>
            <span class="justify-self-start text-3xl font-normal">rpm</span>
          </p>
          <p class="col-span-2 grid w-full grid-cols-subgrid items-end justify-items-end">
            <span class="text-end text-5xl font-light">{volumeFlow()}</span>
            <span class="justify-self-start  text-3xl font-normal">
              m<sup>3</sup>/h
            </span>
          </p>
        </section>
        <input
          type="range"
          min="0"
          class="range"
          max={MAX_SET_POINT}
          value={setPoint() || 0}
          disabled={isDisabled()}
          onInput={handleInput}
          ref={input}
        />
        <ModeFanControl setPoint={setPoint} setSetPoint={setSetPoint} />
      </main>
    </>
  );
}

export default FanControl;
