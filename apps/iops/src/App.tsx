import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./index.css";
import {
  createRouter,
  createHashHistory,
  RouterProvider,
} from "@tanstack/react-router";
import { routeTree } from "./routeTree.gen";
import { ThemeProvider } from "./contexts/theme-context";
import { AuthProvider, useAuth } from "./contexts/auth-context";

const hashHistory = createHashHistory();

const router = createRouter({
  routeTree,
  defaultPreload: "intent",
  scrollRestoration: true,
  history: hashHistory,
  context: {
    auth: undefined!,
  },
});

// Register things for typesafety
declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

function InnerApp() {
  const { auth } = useAuth();
  return (
    <main>
      <RouterProvider router={router} context={{ auth }} />
    </main>
  );
}

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <AuthProvider>
      <ThemeProvider>
        <InnerApp />
      </ThemeProvider>
    </AuthProvider>
  );
}

export default App;
