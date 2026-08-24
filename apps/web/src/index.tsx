import { createRoot } from "react-dom/client";

import "~/style.css";

const App = () => <main></main>;

const root = document.getElementById("root");

if (root) {
  createRoot(root).render(<App />);
}
