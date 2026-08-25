import { Empty } from "antd";

import { Product } from "~/api/products/search.query";
import { CardProduct } from "~/pages/index/components/card-product";

type Props = {
  data: Product[];
};

export const FeedProducts = ({ data }: Props) => {
  return data.length == 0 ? (
    <div className="md:col-span-2 xl:col-span-3">
      <Empty
        image={Empty.PRESENTED_IMAGE_SIMPLE}
        description="No products found"
      />
    </div>
  ) : (
    data.map((product) => (
      <CardProduct data={product} key={product.id} />
    ))
  );
};
