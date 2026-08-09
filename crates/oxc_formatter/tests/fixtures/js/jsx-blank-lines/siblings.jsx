const elements = (
  <>

    <Alpha />

    <Beta />



    <Gamma />
    <Delta />

  </>
);

const expressions = (
  <div>
    {failure && <Failure />}

    <Status />

    {done && (
      <>
        <Result />
        <Track />
      </>
    )}
  </div>
);

// A meaningful text child already drops blank lines under either setting.
const withText = (
  <>
    <Alpha />

    <Beta />

    Third
  </>
);

const nested = (
  <section>
    <header>
      <h1>Title</h1>

      <p>Lede</p>
    </header>

    <footer />
  </section>
);
