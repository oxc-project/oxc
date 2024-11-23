var _toPropertyKey = require("@babel/runtime/helpers/toPropertyKey");
var _objectWithoutProperties = require("@babel/runtime/helpers/objectWithoutProperties");
const input = {};
const _ref = prefix + "state", _ref2 = `${prefix}consents`, { given_name: givenName, "last_name": lastName, [`country`]: country, [_ref]: state, [_ref2]: consents } = input, rest = _objectWithoutProperties(input, [
	"given_name",
	"last_name",
	`country`,
	_ref,
	_ref2
].map(_toPropertyKey));
