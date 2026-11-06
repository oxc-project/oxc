(function () {
  //#region \0rolldown/runtime.js
  var __commonJSMin = (cb, mod) => () => (
    mod || (cb((mod = { exports: {} }).exports, mod), (cb = null)),
    mod.exports
  );
  //#endregion
  //#region ../../npm/runtime/src/helpers/decoratorMetadata.js
  var require_decoratorMetadata = /* @__PURE__ */ __commonJSMin((exports, module) => {
    function _decoratorMetadata(array, target) {
      var symbolMetadata = Symbol.metadata || Symbol["for"]("Symbol.metadata");
      var value = array[3];
      if (symbolMetadata in target)
        Object.defineProperty(target, symbolMetadata, {
          enumerable: true,
          configurable: true,
          writable: true,
          value,
        });
      else target[symbolMetadata] = value;
      return target;
    }
    ((module.exports = _decoratorMetadata),
      (module.exports.__esModule = true),
      (module.exports["default"] = module.exports));
  });
  //#endregion
  //#region ../../npm/runtime/src/helpers/setClassName.js
  var require_setClassName = /* @__PURE__ */ __commonJSMin((exports, module) => {
    function _setClassName(target, name) {
      var desc = Object.getOwnPropertyDescriptor(target, "name");
      if (!desc || ("value" in desc && !desc.writable)) {
        name =
          typeof name === "symbol"
            ? name.description === void 0
              ? ""
              : "[" + name.description + "]"
            : String(name);
        Object.defineProperty(target, "name", {
          value: name,
          configurable: true,
        });
      }
      return target;
    }
    ((module.exports = _setClassName),
      (module.exports.__esModule = true),
      (module.exports["default"] = module.exports));
  });
  //#endregion
  //#region ../../npm/runtime/src/helpers/decorateElement.js
  var require_decorateElement = /* @__PURE__ */ __commonJSMin((exports, module) => {
    var _decoratorMetadata = require_decoratorMetadata();
    var _setClassName = require_setClassName();
    var decoratorStrings = [
      "class",
      "method",
      "getter",
      "setter",
      "accessor",
      "field",
      "value",
      "get",
      "set",
    ];
    function expectFn(fn, message) {
      if (fn !== void 0 && typeof fn !== "function") throw new TypeError(message);
      return fn;
    }
    function decoratorContext(kind, name, done, metadata, fns) {
      return {
        kind: decoratorStrings[kind],
        name,
        metadata,
        addInitializer: function (fn) {
          if (done._)
            throw new TypeError("attempted to call addInitializer after decoration was finished");
          fns.push(expectFn(fn || null, "An initializer must be a function"));
        },
      };
    }
    function privateHas(brand, x) {
      if (Object(x) !== x) throw new TypeError("right-hand side of 'in' should be an object");
      return typeof brand === "function" ? x === brand : brand.has(x);
    }
    function privateAccess(kind, brand, desc) {
      function check(x) {
        if (!privateHas(brand, x))
          throw new TypeError("Private element is not present on this object");
      }
      var access = {
        has: function (x) {
          return privateHas(brand, x);
        },
      };
      if (kind !== 3)
        access.get = function (x) {
          check(x);
          return kind === 5 ? brand.get(x) : kind === 1 ? desc.value : desc.get.call(x);
        };
      if (kind > 2)
        access.set = function (x, value) {
          check(x);
          if (kind === 5) brand.set(x, value);
          else desc.set.call(x, value);
        };
      return access;
    }
    /**
     * Apply decorators to a class element or class.
     *
     * `flags`: kind in bits 0-2 (0 class, 1 method, 2 getter, 3 setter, 4 accessor, 5 field),
     * 8 static, 16 private. Decorators are `[receiver, function]` pairs.
     *
     * `privateInfo` contains a brand and a method/accessor descriptor. Field brands
     * are WeakMaps; method brands are WeakSets or the original static constructor.
     */
    function _decorateElement(array, flags, name, decorators, target, privateInfo) {
      var k = flags & 7,
        s = !!(flags & 8),
        p = !!(flags & 16);
      var j = k > 3 ? array.length + 1 : k ? (s ? 1 : 2) : 0,
        key = decoratorStrings[k + 5];
      var initializers = k > 3 && (array[j - 1] = []),
        extraInitializers = array[j] || (array[j] = []);
      if (k && !s && !p) target = target.prototype;
      var desc = p ? privateInfo[1] : k && k < 5 && Object.getOwnPropertyDescriptor(target, name);
      if (p && k < 5) {
        if (desc.value)
          Object.defineProperty(desc.value, "name", {
            value: name,
            configurable: true,
          });
        if (desc.get)
          Object.defineProperty(desc.get, "name", {
            value: "get " + name,
            configurable: true,
          });
        if (desc.set)
          Object.defineProperty(desc.set, "name", {
            value: "set " + name,
            configurable: true,
          });
      }
      if (!k) {
        name =
          typeof name === "symbol"
            ? name.description === void 0
              ? ""
              : "[" + name.description + "]"
            : String(name);
        _setClassName(target, name);
      }
      for (var i = decorators.length - 1; i >= 0; i--) {
        var done = {},
          ctx = decoratorContext(k, name, done, array[3], extraInitializers),
          it,
          fn;
        if (k) {
          ctx["static"] = s;
          ctx["private"] = p;
          if (p) ctx.access = privateAccess(k, privateInfo[0], desc);
          else {
            var access = (ctx.access = {
              has: function (x) {
                return name in x;
              },
            });
            if (k !== 3)
              access.get = function (x) {
                return x[name];
              };
            if (k > 2)
              access.set = function (x, y) {
                x[name] = y;
              };
          }
        }
        if (typeof decorators[i][1] !== "function")
          throw new TypeError("A decorator must be a function");
        it = Function.prototype.call.call(
          decorators[i][1],
          decorators[i][0],
          k
            ? k < 4
              ? desc[key]
              : k > 4
                ? void 0
                : {
                    get: desc.get,
                    set: desc.set,
                  }
            : target,
          ctx,
        );
        done._ = 1;
        if (k !== 4 || it === void 0) {
          if (
            expectFn(
              it,
              (k === 0 ? "class" : k === 5 ? "field" : "method")
                + " decorators must return a function or undefined",
            )
          )
            k > 4 ? initializers.unshift(it) : k ? (desc[key] = it) : (target = it);
        } else if (typeof it !== "object" || it === null)
          throw new TypeError(
            "accessor decorators must return an object with get, set, or init properties or undefined",
          );
        else {
          if (expectFn((fn = it.get), "accessor.get must be a function")) desc.get = fn;
          if (expectFn((fn = it.set), "accessor.set must be a function")) desc.set = fn;
          if (expectFn((fn = it.init), "accessor.init must be a function"))
            initializers.unshift(fn);
        }
      }
      if (!k) _decoratorMetadata(array, target);
      if (p) return k === 4 ? desc : k < 4 ? desc[key] : void 0;
      if (desc) Object.defineProperty(target, name, desc);
      return target;
    }
    ((module.exports = _decorateElement),
      (module.exports.__esModule = true),
      (module.exports["default"] = module.exports));
  });
  //#endregion
  //#region ../../npm/runtime/src/helpers/decoratorStart.js
  var require_decoratorStart = /* @__PURE__ */ __commonJSMin((exports, module) => {
    /**
     * Create the decorator context array for a class.
     *
     * Slots: 0 class extra initializers, 1 static method extra initializers,
     * 2 instance method extra initializers, 3 metadata object, then
     * `[initializers, extraInitializers]` pairs for each decorated field and accessor.
     */
    function _decoratorStart(base) {
      var symbolMetadata = Symbol.metadata || Symbol["for"]("Symbol.metadata");
      var parentMetadata = base == null ? null : base[symbolMetadata];
      return [, , , Object.create(parentMetadata == null ? null : parentMetadata)];
    }
    ((module.exports = _decoratorStart),
      (module.exports.__esModule = true),
      (module.exports["default"] = module.exports));
  });
  //#endregion
  //#region ../../npm/runtime/src/helpers/runInitializers.js
  var require_runInitializers = /* @__PURE__ */ __commonJSMin((exports, module) => {
    /**
     * Run initializers stored in slot `flags >> 1` of the decorator context array.
     * If `flags & 1`, these are extra initializers which are called for their side effects.
     * Otherwise each initializer transforms `value`, which is returned.
     */
    function _runInitializers(array, flags, self, value) {
      for (var i = 0, fns = array[flags >> 1], n = fns && fns.length; i < n; i++)
        flags & 1 ? fns[i].call(self) : (value = fns[i].call(self, value));
      return value;
    }
    ((module.exports = _runInitializers),
      (module.exports.__esModule = true),
      (module.exports["default"] = module.exports));
  });
  //#endregion
  //#region ../coverage/src/runtime/getDecoratorHelpers.js
  babelHelpers.decorateElement = require_decorateElement();
  babelHelpers.decoratorMetadata = require_decoratorMetadata();
  babelHelpers.decoratorStart = require_decoratorStart();
  babelHelpers.runInitializers = require_runInitializers();
  babelHelpers.setClassName = require_setClassName();
  //#endregion
})();
