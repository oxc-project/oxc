const before={  formatted:true};

// prettier-ignore-start
const prettierRange={  keep:"spaces"  }
const prettierArray=[
1,2,3
]
// prettier-ignore-end

const between={  formatted:true};

/* oxfmt-ignore-start */
function keepThis(  value  ) {
return   {value}
}
/* oxfmt-ignore-end */

const after={  formatted:true};

function nested() {
  const before={  formatted:true};

  // prettier-ignore-start
  const kept={  nested:true}
    call(  kept  )
  // prettier-ignore-end

  const after={  formatted:true};
}

class Demo {
  before={  formatted:true};

  // oxfmt-ignore-start
  kept={  spacing:"raw"  }
  method(  value  ) {return   value}
  // oxfmt-ignore-end

  after={  formatted:true};
}
