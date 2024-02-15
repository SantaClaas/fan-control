import { createResource } from "solid-js";
import Slider from "./Slider";

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
  return <>{JSON.stringify(setPointResponse())}</>;
}

export default App;
