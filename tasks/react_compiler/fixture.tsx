import { useState } from "react";

interface Product {
  id: number;
  name: string;
  price: number;
}

interface ProductListProps {
  products: Product[];
  query: string;
}

export function ProductList({ products, query }: ProductListProps) {
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const visibleProducts = products
    .filter((product) => product.name.toLowerCase().includes(query.toLowerCase()))
    .toSorted((left, right) => left.price - right.price);

  return (
    <ul>
      {visibleProducts.map((product) => (
        <li key={product.id}>
          <button
            aria-pressed={selectedId === product.id}
            onClick={() => setSelectedId(product.id)}
          >
            {product.name}: ${product.price}
          </button>
        </li>
      ))}
    </ul>
  );
}
