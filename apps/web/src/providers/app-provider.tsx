import { QueryClientProvider } from "@tanstack/react-query";
import type { ThemeConfig } from "antd";
import { App as AntdApp, ConfigProvider, theme as antdTheme } from "antd";
import type { PropsWithChildren } from "react";

import { queryClient } from "~/api";

export const AppProvider = ({ children }: PropsWithChildren) => {
  const industrialTheme = {
    algorithm: antdTheme.defaultAlgorithm,
    token: {
      ...antdTheme.useToken().token,
      borderRadius: 10,
      borderRadiusLG: 15,
      borderRadiusSM: 5,
      colorBgBase: "#f1f5f9",
      colorBgContainer: "#ffffff",
      colorBgElevated: "#f8fafc",
      fontFamily: "IBM Plex Sans, sans-serif",
    },
  } satisfies ThemeConfig;

  return (
    <ConfigProvider
      componentSize="medium"
      theme={industrialTheme}
      variant="outlined"
    >
      <AntdApp component={false}>
        <QueryClientProvider client={queryClient}>
          {children}
        </QueryClientProvider>
      </AntdApp>
    </ConfigProvider>
  );
};
