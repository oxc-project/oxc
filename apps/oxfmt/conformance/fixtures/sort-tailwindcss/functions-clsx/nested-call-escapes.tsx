const A = <div className={clsx(
  "flex",
  str.includes("\t") && "p-4",
  str.match(/\s/) && "m-2"
)} />;