class C {
  #fn = function () { return this; };
  run() { return (this?.#fn!)(); }
  runOptional() { return (this?.#fn!)?.(); }
  runMultiple() { return (this?.#fn!!)(); }
  runOptionalMultiple() { return (this?.#fn!!!)?.(); }
}
