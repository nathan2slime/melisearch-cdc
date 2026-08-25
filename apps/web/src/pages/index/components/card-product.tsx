import { Card, Tag, Typography } from "antd";

import type { Product } from "~/api/products/search.query";

type Props = {
  data: Product;
};

const priceFormatter = new Intl.NumberFormat("pt-BR", {
  style: "currency",
  currency: "BRL",
});

export const CardProduct = ({ data }: Props) => {
  return (
    <Card
      className="h-full cursor-pointer"
      cover={
        <div className="bg-linear-to-r from-blue-50 p-6 to-indigo-50 rounded-xl h-40">
          <Tag color="green-inverse" variant="outlined">
            {priceFormatter.format(data.price_cents / 100)}
          </Tag>
        </div>
      }
    >
      <Card.Meta
        title={data.name}

        description={
          <div>
            <Typography.Paragraph className="line-clamp-3">
              {data.description}
            </Typography.Paragraph>
          </div>
        }
      />
    </Card>
  );
};
