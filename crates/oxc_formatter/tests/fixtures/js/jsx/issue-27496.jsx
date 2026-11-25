// A comment-only expression container is a group like Prettier's;
// measuring the over-long comment resets the re-measure pending from the flat hard line after `text`,
// so `<a>` stays flat and only its `{" "}` separator breaks.
export const X = () => (
  <p>
    text
    {/* this comment is deliberately longer than the print width so that it cannot fit on one line at all */}
    aaa bbb ccc ddd eee fff ggg hhh iii jjj kkk lll <a onClick={() => doSomething()}>link text</a> more
  </p>
);

// A short comment fits, the hard line after it prints flat and `<a>` is re-measured with its separator.
export const Y = () => (
  <p>
    text
    {/* short */}
    aaa bbb ccc ddd eee fff ggg hhh iii jjj kkk lll <a onClick={() => doSomething()}>link text</a> more
  </p>
);
