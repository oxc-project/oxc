// The object and key of a compound assignment or update on a member are
// evaluated once.
function Component({events}) {
  const calls = {keys: 0, targets: 0};
  const keyOf = event => {
    calls.keys += 1;
    return event.key;
  };
  const totals = {a: 0, b: 0};
  const getTotals = () => {
    calls.targets += 1;
    return totals;
  };
  for (const event of events) {
    totals[keyOf(event)] += event.amount;
  }
  getTotals().a *= 10;
  getTotals()[keyOf({key: 'b'})]++;
  return (
    <div>
      {calls.keys}|{calls.targets}|{totals.a}|{totals.b}
    </div>
  );
}

export const FIXTURE_ENTRYPOINT = {
  fn: Component,
  params: [
    {
      events: [
        {key: 'a', amount: 1},
        {key: 'b', amount: 2},
      ],
    },
  ],
};
