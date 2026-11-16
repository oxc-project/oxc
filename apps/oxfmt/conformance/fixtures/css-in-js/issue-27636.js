styled.div`
  .a {
    width: 100%;
    // h: ${(p) => p}px;
  }
`;
styled.div`
  .a {
    width: 100%;
    /* h: ${(p) => p}px; */
  }
`;
styled.div`
  .a {
    width: 100%;
    // h: ${f(p)} ${a + b} ${a ? b : c} ${[a]} ${{ a }};
  }
`;
styled.div`
  .a {
    width: 100%;
    // h: ${(p) => p.theme.someVeryLongPropertyName.anotherLongPropertyName.yetAnother.andMore}px;
  }
`;
styled.div`
  .a {
    width: 100%;
    // h: ${(p) =>
      p}px;
  }
`;
styled.div`
  .a {
    width: 100%;
    // h: ${function (p) { return p; }}px;
  }
`;
