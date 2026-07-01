import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./index.css";
import Login from "./features/login";
import {
  createRouter,
  createHashHistory,
  RouterProvider,
} from "@tanstack/react-router";
import { routeTree } from "./routeTree.gen";
import { ThemeProvider } from "./contexts/theme-context";

const hashHistory = createHashHistory();

const router = createRouter({
  routeTree,
  defaultPreload: "intent",
  scrollRestoration: true,
  history: hashHistory,
});

// Register things for typesafety
declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <main>
      <ThemeProvider>
        <RouterProvider router={router} />
      </ThemeProvider>
    </main>
  );
}

export default App;
