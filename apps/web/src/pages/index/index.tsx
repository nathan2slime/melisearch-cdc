import { useSuspenseQuery } from "@tanstack/react-query";
import { Input, Pagination, Spin } from "antd";
import { SearchProps } from "antd/es/input";
import { useState } from "react";
import { Loading3QuartersOutlined, SearchOutlined } from "@ant-design/icons";

import {
  searchProducts,
  type SearchProductsParams,
} from "~/api/products/search.query";
import { FeedProducts } from "~/pages/index/components/feed-products";

type IndexProps = {
  args: SearchProductsParams;
  onSearchChange: (search: Partial<SearchProductsParams>) => Promise<void>;
};

const MAX_SEARCH_TOTAL_HITS = 1_000;

export const Index = ({ args: search, onSearchChange }: IndexProps) => {
  const [searchText, setSearchText] = useState(search.q ?? "");

  const { data: products, isFetching } = useSuspenseQuery({
    ...searchProducts(search),
  });
  const paginationTotal = products
    ? products.total_pages * products.per_page
    : search.per_page;

  const handleChangeSearchText: SearchProps["onChange"] = (e) =>
    setSearchText(e.target.value);

  const handleChangeSearch: SearchProps["onSearch"] = (q) =>
    onSearchChange({ ...search, q, page: 1 });

  const handleChangePagination = (page: number, perPage: number) =>
    onSearchChange({ ...search, page, per_page: perPage });

  return (
    <div className="p-8 flex flex-col justify-center items-center">
      <div className="flex justify-end w-full items-center gap-2 mb-5">
        <Input.Search
          allowClear
          enterButton={<SearchOutlined />}
          className="max-w-xs"
          placeholder="Search products by name"
          value={searchText}
          onChange={handleChangeSearchText}
          onSearch={handleChangeSearch}
        />
      </div>

      <div className="mb-4 grid grid-cols-1 gap-4 md:grid-cols-2 xl:grid-cols-6">
        {isFetching ? (
          <Spin
            indicator={<Loading3QuartersOutlined />}
            className="animate-spin"
            size="large"
          />
        ) : (
          <FeedProducts data={products.items} />
        )}
      </div>

      <div className="flex justify-end w-full">
        <Pagination
          current={search.page}
          align="center"
          showTotal={(total) => `${total} product(s)`}
          pageSize={search.per_page}
          pageSizeOptions={[10, 20, 50, 100]}
          showSizeChanger
          total={
            paginationTotal >= MAX_SEARCH_TOTAL_HITS
              ? MAX_SEARCH_TOTAL_HITS
              : paginationTotal
          }
          onChange={handleChangePagination}
        />
      </div>
    </div>
  );
};
