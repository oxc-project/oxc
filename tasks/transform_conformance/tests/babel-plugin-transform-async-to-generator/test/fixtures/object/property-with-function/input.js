const Normal = {
  foo: async () => {
    console.log(log)
  }
}

const StringLiteralKey = {
  ['bar']: async () => {
  }
}

const EmptyStringLiteralKey = {
  ['']: async () => {
    console.log(this)
  }
}

const InvalidStringLiteralKey = {
  ['#']: async () => {},
  ['this']: async () => {},
  ['#default']: async () => {},
  ['O X C']: async () => {}
}
