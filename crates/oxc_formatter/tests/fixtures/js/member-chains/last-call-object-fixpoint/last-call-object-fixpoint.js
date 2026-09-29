// DIVERGENCES.md#member-chain-last-call-object-fixpoint
// https://github.com/oxc-project/oxc/issues/27053
const { error } = await client.from("sent_messages").insert({ id: sid, account_id: accountId, subject: "x", body: "x", message_id: id, recipient_count: count });

// https://github.com/oxc-project/oxc/issues/23852
const fetchMock = vi.fn().mockResolvedValue(res({ success: true, data: [{ roomTypeID: "rt1", rooms: [{ roomID: "u1", roomName: "101", roomType: "single" }] }] }));

// The split layout keeps the objects flat
const { error } = await client.from("sent_messages").insert([{ id: sid, account_id: accountId, subject: "x", body: "x", message_id: id }]);

const fetchCheckout = vi.fn().mockResolvedValue(new Response(JSON.stringify(listing), { status: 200 }));

// An object in a template literal does not break
streams.get(endpoint)?.enqueue(encoder.encode(`event: message\ndata: ${JSON.stringify({ jsonrpc: "2.0", id: message.id, result })}\n\n`));

// The split layout breaks the object
const { error } = await client.from("sent_messages").insert(await make({ id: sid, account_id: accountId, subject: "x", body: "x", message_id: id }));

model = types.model({ something: mxSomething }).volatile((self) => ({ loading: false, savingStatus: "idle", undoDisabled: false, aiFocused: false, online: true }));

// The object breaks only in the split layout, it fits in the one line layout
it("accepts an account endpoint under the configured data-residency tenant", async () => {
  it("nested", async () => {
    const fetchImpl = vi.fn().mockResolvedValue(Response.json({ endpoints: { api: "https://copilot-api.acme.ghe.com" } }));
  });
});

// Every line of the split layout fits
d3.select("body").append("circle").at({ width: 30, fill: "#f0f" }).st({ fontWeight: 600, color: "red" });

// The first line of the one line layout does not fit
const { error } = await someVeryLongClientNameForTheDatabase.from("sent_messages_with_a_long_table_name").insert({ id: sid, account_id: accountId, subject: "x", body: "x", message_id: id, recipient_count: count });

// A pending line suffix when trying the split layout
const x = // comment
  client.from("sent_messages").insert({ id: sid, account_id: accountId, subject: "x", body: "x", message_id: id, recipient_count: count });

const y = client.from("sent_messages").insert({ id: sid, account_id: accountId, subject: "x", body: "x", message_id: id }, // after the argument
);
