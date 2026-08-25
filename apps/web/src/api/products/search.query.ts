import { queryOptions } from "@tanstack/react-query";

import { api } from "~/api";

export type Product = {
  id: number;
  name: string;
  description: string | null;
  price_cents: number;
  stock: number;
};

export type SearchProductsParams = {
  q?: string;
  page: number;
  per_page: number;
};

export type SearchProductsResponse = {
  items: Product[];
  page: number;
  per_page: number;
  total_items: number;
  total_pages: number;
};

export const searchProducts = ({
  q = "",
  page = 1,
  per_page = 20,
}: SearchProductsParams) => {
  const query = q.trim();

  return queryOptions({
    queryKey: ["products", "search", { q: query, page, per_page }],
    queryFn: async () => {
      const response = await api.get<SearchProductsResponse>("/products", {
        params: {
          q: query || undefined,
          page,
          per_page,
        },
      });

      return response.data;
    },
  });
};
