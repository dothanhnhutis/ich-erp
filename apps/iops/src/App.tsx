import "./index.css";
import {
  createRouter,
  createHashHistory,
  RouterProvider,
} from "@tanstack/react-router";
import { routeTree } from "./routeTree.gen";
import { ThemeProvider } from "./contexts/theme-context";
import { AuthProvider, useAuth } from "./contexts/auth-context";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const hashHistory = createHashHistory();
const queryClient = new QueryClient();

const router = createRouter({
  routeTree,
  history: hashHistory,
  context: {
    state: undefined!,
    queryClient,
  },

  defaultPreload: "intent",
  // Since we're using React Query, we don't want loader calls to ever be stale
  // This will ensure that the loader is always called when the route is preloaded or visited
  defaultPreloadStaleTime: 0,
  scrollRestoration: true,
});

// Register things for typesafety
declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

function InnerApp() {
  const { state } = useAuth();
  if (state?.hydrating) return null;

  return (
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} context={{ state }} />
    </QueryClientProvider>
  );
}

function App() {
  return (
    <main>
      <AuthProvider>
        <ThemeProvider>
          <InnerApp />
        </ThemeProvider>
      </AuthProvider>
    </main>
  );
}

export default App;
