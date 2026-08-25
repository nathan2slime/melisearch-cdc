import { createRouter, RouterProvider } from "@tanstack/react-router";
import { setupRouterSsrQueryIntegration } from "@tanstack/react-router-ssr-query";

import { queryClient } from "~/api";
import { AppProvider } from "~/providers/app-provider";
import { routeTree } from "~/routeTree.gen";

export const getRouter = () => {
  const router = createRouter({
    routeTree,
    context: { queryClient },
  });

  setupRouterSsrQueryIntegration({
    router,
    queryClient,
    hydrateOptions: {
      defaultOptions: {
        queries: {
          gcTime: 5 * 60 * 1000,
        },
      },
    },
  });

  return router;
};

declare module "@tanstack/react-router" {
  interface Register {
    router: ReturnType<typeof getRouter>;
  }
}

const router = getRouter();

export const App = () => (
  <AppProvider>
    <RouterProvider router={router} />
  </AppProvider>
);
