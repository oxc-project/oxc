// Sloppy mode

try {} catch ({let}) {}
try {} catch ([let]) {}
try {} catch ({x: let}) {}
try {} catch ({...let}) {}
try {} catch ([...let]) {}
try {} catch ({x: {y: [{...let}]}}) {}
