import { startTransition } from "solid-js";
import styles from "./Slider.module.css";

/**
 * A slider range input based on https://codepen.io/bgebelein/pen/wvYeapy
 * @returns {import("solid-js").JSX.Element}
 */
function Slider() {
  return (
    <>
      <div class={styles.wrapper}>
        <input
          type="range"
          class={styles.slider}
          min="0"
          max="100"
          value="50"
        />
      </div>
    </>
  );
}

export default Slider;
