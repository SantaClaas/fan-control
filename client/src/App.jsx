import { createResource } from "solid-js";
import SliderVertical from "./SliderVertical";

async function getCurrentSetPoint() {
  const response = await fetch("/api/fan/2/setpoint/current");

  if (!response.ok) {
    return { error: "Failed to fetch setpoint" };
  }

  const data = await response.json();
  return data;
}

function App() {
  const [setPointResponse] = createResource(getCurrentSetPoint);
  return (
    <>
      {/* <Slider /> */}
      <SliderVertical />
    </>
  );
}

export default App;
