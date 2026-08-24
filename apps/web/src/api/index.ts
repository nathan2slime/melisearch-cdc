import { QueryClient } from "@tanstack/react-query";
import axios from "axios";
import { message } from "antd";

export const queryClient = new QueryClient();

export const api = axios.create({
  baseURL: process.env.REACT_APP_PUBLIC_API_URL ?? "/api",
});

api.interceptors.response.use(
  (response) => response,
  (error) => {
    message.error(error.response?.data?.message ?? "Something went wrong.");
    
    return Promise.reject(error);
  },
);
