import { createRouter, RouterProvider } from "@tanstack/react-router";
import { AppProvider } from "~/providers/app-provider";
import { routeTree } from "~/routeTree.gen";

const router = createRouter({ routeTree });

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

export const App = () => (
  <AppProvider>
    <RouterProvider router={router} />
  </AppProvider>
);
