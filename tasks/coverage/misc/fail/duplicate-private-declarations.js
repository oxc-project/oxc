class DuplicateField { #x; x; #x; }

class DuplicateGetter { get #x() {} set #x(value) {} get #x() {} }

class DuplicateSetter { set #x(value) {} get #x() {} set #x(value) {} }
