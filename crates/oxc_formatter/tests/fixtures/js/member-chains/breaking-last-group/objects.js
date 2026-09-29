// DIVERGENCES.md#member-chain-breaking-last-group
// oxc#27053: the first pass must not split a chain that rejoins on the next pass.
const { error } = await client.from("sent_messages").insert({ id: sid, account_id: accountId, subject: "x", body: "x", message_id: id, recipient_count: count });

// An authored multiline object follows the same chain layout.
const saved = client.from("table").insert({
  id: sid,
});

// oxc#23852: a call wrapping an object has the same problem.
const fetchMock = vi.fn().mockResolvedValue(res({ success: true, data: [{ roomTypeID: "rt1", rooms: [{ roomID: "u1", roomName: "101", roomType: "single" }] }] }));

const awaited = client.from("sent_messages").insert(await make({ id: sid, account_id: accountId, subject: "x", body: "x", message_id: id }));
const array = client.from("sent_messages").insert([{ id: sid, account_id: accountId, subject: "x", body: "x", message_id: id }]);
const response = vi.fn().mockResolvedValue(new Response(JSON.stringify(listing), { status: 200 }));

// Indentation can make the split layout break an otherwise fitting object.
function outer() {
  function inner() {
    const fetchImpl = vi.fn().mockResolvedValue(Response.json({ endpoints: { api: "https://copilot-api.acme.ghe.com" } }));
  }
}

// A chain whose callee line itself exceeds the width stays split.
const result = someVeryLongClientNameForTheDatabase.from("sent_messages_with_a_long_table_name").insert({ id: sid, account_id: accountId, subject: "x", body: "x", message_id: id });
