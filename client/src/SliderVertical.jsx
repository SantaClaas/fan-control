import { createEffect, createSignal } from "solid-js";

/**
 * A slider range input based on https://codepen.io/bgebelein/pen/wvYeapy
 * @returns {import("solid-js").JSX.Element}
 */
function SliderVertical() {
  const [rpm, setRpm] = createSignal(0);
  let input;
  createEffect(() => {
    if (input) input.value = rpm();
  });
  return (
    <>
      <label class="flex gap-2 p-3">
        <span class="align-middle">RPM</span>
        <input
          type="range"
          class="flex-grow"
          min="0"
          max="100"
          value="50"
          onChange={(event) => setRpm(Number(event.target.value))}
          ref={input}
        />
        <span class="align-middle min-w-[4ch]">{rpm()}</span>
      </label>
    </>
  );
}

export default SliderVertical;
