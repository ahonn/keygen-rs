/******************************************************************************
Copyright (c) Microsoft Corporation.

Permission to use, copy, modify, and/or distribute this software for any
purpose with or without fee is hereby granted.

THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH
REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY
AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT,
INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES WHATSOEVER RESULTING FROM
LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT, NEGLIGENCE OR
OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR
PERFORMANCE OF THIS SOFTWARE.
***************************************************************************** */
/* global Reflect, Promise, SuppressedError, Symbol */

var extendStatics = function(d, b) {
  extendStatics = Object.setPrototypeOf ||
      ({ __proto__: [] } instanceof Array && function (d, b) { d.__proto__ = b; }) ||
      function (d, b) { for (var p in b) if (Object.prototype.hasOwnProperty.call(b, p)) d[p] = b[p]; };
  return extendStatics(d, b);
};

function __extends(d, b) {
  if (typeof b !== "function" && b !== null)
      throw new TypeError("Class extends value " + String(b) + " is not a constructor or null");
  extendStatics(d, b);
  function __() { this.constructor = d; }
  d.prototype = b === null ? Object.create(b) : (__.prototype = b.prototype, new __());
}

function __awaiter(thisArg, _arguments, P, generator) {
  function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
  return new (P || (P = Promise))(function (resolve, reject) {
      function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
      function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
      function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
      step((generator = generator.apply(thisArg, _arguments || [])).next());
  });
}

function __generator(thisArg, body) {
  var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
  return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
  function verb(n) { return function (v) { return step([n, v]); }; }
  function step(op) {
      if (f) throw new TypeError("Generator is already executing.");
      while (g && (g = 0, op[0] && (_ = 0)), _) try {
          if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
          if (y = 0, t) op = [op[0] & 2, t.value];
          switch (op[0]) {
              case 0: case 1: t = op; break;
              case 4: _.label++; return { value: op[1], done: false };
              case 5: _.label++; y = op[1]; op = [0]; continue;
              case 7: op = _.ops.pop(); _.trys.pop(); continue;
              default:
                  if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                  if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                  if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                  if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                  if (t[2]) _.ops.pop();
                  _.trys.pop(); continue;
          }
          op = body.call(thisArg, _);
      } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
      if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
  }
}

typeof SuppressedError === "function" ? SuppressedError : function (error, suppressed, message) {
  var e = new Error(message);
  return e.name = "SuppressedError", e.error = error, e.suppressed = suppressed, e;
};

// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
/** @ignore */
function uid() {
    return window.crypto.getRandomValues(new Uint32Array(1))[0];
}
/**
 * Transforms a callback function to a string identifier that can be passed to the backend.
 * The backend uses the identifier to `eval()` the callback.
 *
 * @return A unique identifier associated with the callback function.
 *
 * @since 1.0.0
 */
function transformCallback(callback, once = false) {
    const identifier = uid();
    const prop = `_${identifier}`;
    Object.defineProperty(window, prop, {
        value: (result) => {
            if (once) {
                Reflect.deleteProperty(window, prop);
            }
            return callback === null || callback === void 0 ? void 0 : callback(result);
        },
        writable: false,
        configurable: true
    });
    return identifier;
}
/**
 * Sends a message to the backend.
 * @example
 * ```typescript
 * import { invoke } from '@tauri-apps/api/tauri';
 * await invoke('login', { user: 'tauri', password: 'poiwe3h4r5ip3yrhtew9ty' });
 * ```
 *
 * @param cmd The command name.
 * @param args The optional arguments to pass to the command.
 * @return A promise resolving or rejecting to the backend response.
 *
 * @since 1.0.0
 */
async function invoke(cmd, args = {}) {
    return new Promise((resolve, reject) => {
        const callback = transformCallback((e) => {
            resolve(e);
            Reflect.deleteProperty(window, `_${error}`);
        }, true);
        const error = transformCallback((e) => {
            reject(e);
            Reflect.deleteProperty(window, `_${callback}`);
        }, true);
        window.__TAURI_IPC__({
            cmd,
            callback,
            error,
            ...args
        });
    });
}

var KeygenError = /** @class */ (function (_super) {
    __extends(KeygenError, _super);
    function KeygenError(code, detail) {
        var _this = _super.call(this, "Keygen error: ".concat(code, " - ").concat(detail)) || this;
        _this.code = code;
        _this.detail = detail;
        _this.name;
        return _this;
    }
    return KeygenError;
}(Error));
var NO_MACHINE_ERROR_CODES = ['NO_MACHINE', 'NO_MACHINES', 'FINGERPRINT_SCOPE_MISMATCH'];
function isInvokeError(err) {
    return typeof err === 'object' && (err === null || err === void 0 ? void 0 : err.hasOwnProperty('code'));
}
function getLicense() {
    return __awaiter(this, void 0, void 0, function () {
        var license, err_1, code, detail;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    _a.trys.push([0, 2, , 3]);
                    return [4 /*yield*/, invoke('plugin:keygen-rs|get_license')];
                case 1:
                    license = _a.sent();
                    return [2 /*return*/, license];
                case 2:
                    err_1 = _a.sent();
                    if (isInvokeError(err_1)) {
                        code = err_1.code, detail = err_1.detail;
                        throw new KeygenError(code, detail);
                    }
                    throw new KeygenError('ERROR', err_1.message);
                case 3: return [2 /*return*/];
            }
        });
    });
}
function validateKey(key, entitlements) {
    return __awaiter(this, void 0, void 0, function () {
        var license, err_2, code, detail, noMachineError, license;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    _a.trys.push([0, 2, , 6]);
                    return [4 /*yield*/, invoke('plugin:keygen-rs|validate_key', {
                            key: key,
                            entitlements: entitlements,
                        })];
                case 1:
                    license = _a.sent();
                    return [2 /*return*/, license];
                case 2:
                    err_2 = _a.sent();
                    if (!isInvokeError(err_2)) return [3 /*break*/, 5];
                    code = err_2.code, detail = err_2.detail;
                    noMachineError = NO_MACHINE_ERROR_CODES.includes(code);
                    if (!noMachineError) {
                        throw new KeygenError(code, detail);
                    }
                    return [4 /*yield*/, invoke('plugin:keygen-rs|activate', {})];
                case 3:
                    _a.sent();
                    return [4 /*yield*/, invoke('plugin:keygen-rs|validate_key', {
                            key: key,
                            entitlements: entitlements,
                        })];
                case 4:
                    license = _a.sent();
                    return [2 /*return*/, license];
                case 5: throw new KeygenError('ERROR', err_2.message);
                case 6: return [2 /*return*/];
            }
        });
    });
}
function deactivate() {
    return __awaiter(this, void 0, void 0, function () {
        var err_3, code, detail;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    _a.trys.push([0, 2, , 3]);
                    return [4 /*yield*/, invoke('plugin:keygen-rs|deactivate')];
                case 1:
                    _a.sent();
                    return [3 /*break*/, 3];
                case 2:
                    err_3 = _a.sent();
                    if (isInvokeError(err_3)) {
                        code = err_3.code, detail = err_3.detail;
                        throw new KeygenError(code, detail);
                    }
                    throw new KeygenError('ERROR', err_3.message);
                case 3: return [2 /*return*/];
            }
        });
    });
}
function checkout(ttl, include) {
    return __awaiter(this, void 0, void 0, function () {
        var err_4, code, detail;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    _a.trys.push([0, 2, , 3]);
                    return [4 /*yield*/, invoke('plugin:keygen-rs|checkout_license', {
                            ttl: ttl,
                            include: include,
                        })];
                case 1:
                    _a.sent();
                    return [3 /*break*/, 3];
                case 2:
                    err_4 = _a.sent();
                    if (isInvokeError(err_4)) {
                        code = err_4.code, detail = err_4.detail;
                        throw new KeygenError(code, detail);
                    }
                    throw new KeygenError('ERROR', err_4.message);
                case 3: return [2 /*return*/];
            }
        });
    });
}

export { KeygenError, checkout, deactivate, getLicense, validateKey };
