import { startTransition } from "solid-js";
import styles from "./Slider.module.css";

function Slider() {
  return <input class={styles.slider} type="range" />;
}

export default Slider;
