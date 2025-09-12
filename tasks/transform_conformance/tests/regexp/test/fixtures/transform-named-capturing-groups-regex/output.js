c1 = new RegExp("(?<a>b)", "");
c2 = new RegExp("((?<a>b)){2}", "");

nested1 = new RegExp("(?<!(?<a>b))", "");
nested2 = new RegExp("((?<a>b))", "");
nested3 = new RegExp("(?:(?<a>b))", "");
