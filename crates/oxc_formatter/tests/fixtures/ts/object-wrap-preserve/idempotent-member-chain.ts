// An object that cannot fit the line width breaks on the first pass and leaves a newline after
// `{`. With `objectWrap: "preserve"` a second pass would read that as an authored expansion and
// the enclosing member chain could choose a different layout. Pre-expanding keeps both passes equal.
document.querySelector("#load")?.addEventListener("click", () => service.load(
  { ...(params.has("page") ? { startingPage: params.get("page") } : {}), ...(params.get("extra") === "1" ? { showMetadata: true, reserveActionSpace: true } : {}) },
  params.get("mode") ?? "fresh",
).catch((error) => console.error(error)));
