var _objectWithoutProperties = require("@babel/runtime/helpers/objectWithoutProperties");
const input = {};
const { given_name: givenName, "last_name": lastName, [`country`]: country } = input, rest = _objectWithoutProperties(input, [
	"given_name",
	"last_name",
	`country`
]);
