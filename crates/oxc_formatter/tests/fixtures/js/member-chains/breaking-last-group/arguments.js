// DIVERGENCES.md#member-chain-breaking-last-group
// Comments must survive either layout, including a pending line suffix.
const x = // comment
  client.from("sent_messages").insert({ id: sid, account_id: accountId, subject: "x", body: "x", message_id: id });
const y = client.from("sent_messages").insert({ id: sid, account_id: accountId, subject: "x", body: "x", message_id: id }, // after the argument
);
const z = client.from("table").insert({
  // before the property
  id: sid, // after the property
}, options);

// All argument positions, and nested member chains.
client.from("table").insert(options, { id: sid, account_id: accountId, subject: "x", body: "x", message_id: id });
client.from("table").insert(...make({ id: sid, account_id: accountId, subject: "x", body: "x", message_id: id }));
const nested = client.from("table").insert(other.from("table").insert({ id: sid, account_id: accountId, subject: "x", body: "x", message_id: id }));

// Multiline callbacks follow the same rule as a multiline object.
const callback = client.from("table").insert(function () {
  return value;
});
const arrow = client.from("table").insert(() => {
  return value;
});
// Literal newlines do not force the enclosing group to expand.
const literal = client.from("table").insert(`first
second`);

// Short flat chains and merged heads still fit.
const short = client.from("table").insert({ id: sid });
const merged = factory().insert({
  id: sid,
});
const template = streams.get(endpoint)?.enqueue(encoder.encode(`data: ${JSON.stringify({ jsonrpc: "2.0", id: message.id, result })}`));

// prettier-ignore
const ignored = client.from("table").insert({
 id: sid
});
