import { useQuery } from "@tanstack/react-query";

function Component() {
  // This should trigger the exhaustive-deps rule
  const { data } = useQuery({
    queryKey: ["todos"],
    queryFn: () => fetch("/api/todos"),
  });

  return <div>{data}</div>;
}
