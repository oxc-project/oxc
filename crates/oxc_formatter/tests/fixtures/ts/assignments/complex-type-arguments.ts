// Type argument is a `TSMappedType`
const emitter = createGlobalEmitter<{
  [key in Event["type"]]: Extract<Event, { type: key }>
}>()

// Type argument is a `TSTypeLiteral`
const emitter2 = createGlobalEmitter<{
  longlonglonglongKey: Extract<Event, { type: key }>
}>()
