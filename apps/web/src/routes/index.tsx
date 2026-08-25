import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { z } from "zod";

import {
  searchProducts,
  type SearchProductsParams,
} from "~/api/products/search.query";
import { Index } from "~/pages/index";

const DEFAULT_PAGE = 1;
const DEFAULT_PER_PAGE = 20;
const MAX_PER_PAGE = 100;
const MAX_SEARCH_TOTAL_HITS = 1_000;

const maxSearchPage = (perPage: number) =>
  Math.max(DEFAULT_PAGE, Math.ceil(MAX_SEARCH_TOTAL_HITS / perPage));

const productsSearchSchema = z
  .object({
    q: z
      .string()
      .trim()
      .transform((value) => value || undefined)
      .optional()
      .catch(undefined),
    page: z.coerce.number().int().min(1).catch(DEFAULT_PAGE),
    per_page: z.coerce
      .number()
      .int()
      .min(1)
      .max(MAX_PER_PAGE)
      .catch(DEFAULT_PER_PAGE),
  })
  .transform((search) => ({
    ...search,
    page: Math.min(search.page, maxSearchPage(search.per_page)),
  })) satisfies z.ZodType<SearchProductsParams>;

const Page = () => {
  const search = Route.useSearch();
  const navigate = useNavigate({ from: Route.fullPath });

  const updateSearch = async (nextSearch: Partial<SearchProductsParams>) => {
    await navigate({
      search: (currentSearch) => ({
        q: nextSearch.q ?? currentSearch.q,
        page: nextSearch.page ?? currentSearch.page,
        per_page: nextSearch.per_page ?? currentSearch.per_page,
      }),
    });
  };

  return <Index args={search} onSearchChange={updateSearch} />;
};

export const Route = createFileRoute("/")({
  validateSearch: productsSearchSchema,
  loaderDeps: ({ search }) => search,
  loader: ({ context, deps }) =>
    context.queryClient.query(searchProducts(deps)),
  component: Page,
});
