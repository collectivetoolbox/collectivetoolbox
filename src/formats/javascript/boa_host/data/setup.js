
        globalThis.setTimeout = function(cb, ms, ...args) { cb(...args); return 0; };
        globalThis.clearTimeout = function(id) {};
        globalThis.setInterval = function(cb, ms, ...args) { cb(...args); return 0; };
        globalThis.clearInterval = function(id) {};
        globalThis.setImmediate = function(cb, ...args) { cb(...args); return 0; };
        globalThis.clearImmediate = function(id) {};

        globalThis.process = {
          platform: "linux",
          argv: ["boa", "tsc.js"],
          env: {},
          cwd() { return globalThis.__rust_cwd(); },
          nextTick(cb) { cb(); },
          stdout: {
            write(data) { globalThis.__rust_print_stdout(data); },
            columns: 80,
            isTTY: false
          },
          exit(code) { throw new Error("process.exit: " + code); },
          execArgv: [],
          pid: 12345,
          memoryUsage() { return { heapUsed: 0 }; }
        };

        const base64Digits = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";
        const btoa = globalThis.btoa || function(str) {
          let result = "";
          let i = 0;
          while (i < str.length) {
            const byte1 = str.charCodeAt(i++) || 0;
            const byte2 = str.charCodeAt(i++) || 0;
            const byte3 = str.charCodeAt(i++) || 0;
            const enc1 = byte1 >> 2;
            const enc2 = ((byte1 & 3) << 4) | (byte2 >> 4);
            let enc3 = ((byte2 & 15) << 2) | (byte3 >> 6);
            let enc4 = byte3 & 63;
            if (isNaN(byte2)) { enc3 = enc4 = 64; }
            else if (isNaN(byte3)) { enc4 = 64; }
            result += base64Digits.charAt(enc1) + base64Digits.charAt(enc2) + base64Digits.charAt(enc3) + base64Digits.charAt(enc4);
          }
          return result;
        };

        const atob = globalThis.atob || function(input) {
          let result = "";
          let i = 0;
          while (i < input.length) {
            const enc1 = base64Digits.indexOf(input.charAt(i++));
            const enc2 = base64Digits.indexOf(input.charAt(i++));
            const enc3 = base64Digits.indexOf(input.charAt(i++));
            const enc4 = base64Digits.indexOf(input.charAt(i++));
            const byte1 = (enc1 << 2) | (enc2 >> 4);
            const byte2 = ((enc2 & 15) << 4) | (enc3 >> 2);
            const byte3 = ((enc3 & 3) << 6) | enc4;
            result += String.fromCharCode(byte1);
            if (enc3 !== 64) { result += String.fromCharCode(byte2); }
            if (enc4 !== 64) { result += String.fromCharCode(byte3); }
          }
          return result;
        };

        globalThis.Buffer = {
          from(input, encoding) {
            if (typeof input === "string") {
              return {
                length: input.length,
                toString(enc) {
                  if (encoding === "base64" && enc === "utf8") {
                    return atob(input);
                  }
                  if (enc === "base64") {
                    return btoa(input);
                  }
                  return input;
                }
              };
            }
            return {
              length: input.length || 0,
              toString(enc) { return ""; }
            };
          }
        };

        const realpathSync = function(path) { return globalThis.__rust_path_resolve(path); };
        realpathSync.native = realpathSync;

        const builtinModules = {
          fs: {
              readFileSync(path, encoding) {
                return globalThis.__rust_read_file(path, encoding);
              },
              writeFileSync(path, data) {
                globalThis.__rust_write_file(path, data);
              },
              existsSync(path) {
                return globalThis.__rust_file_exists(path);
              },
              readdirSync(path) {
                return globalThis.__rust_read_dir(path);
              },
              statSync(path) {
                if (!globalThis.__rust_file_exists(path)) {
                  const err = new Error("ENOENT: no such file or directory, stat '" + path + "'");
                  err.code = "ENOENT";
                  throw err;
                }
                const isFile = globalThis.__rust_is_file(path);
                const sizeStr = globalThis.__rust_file_size(path);
                const mtimeStr = globalThis.__rust_file_mtime(path);
                return {
                  size: Number(sizeStr),
                  mtime: new Date(Number(mtimeStr)),
                  isFile() { return isFile; },
                  isDirectory() { return !isFile; },
                  isSymbolicLink() { return false; }
                };
              },
              openSync(path, flags) {
                const fd = (globalThis.__next_fd || 1) + 1;
                globalThis.__next_fd = fd;
                globalThis.__fds = globalThis.__fds || {};
                globalThis.__fds[fd] = path;
                return fd;
              },
              writeSync(fd, data) {
                const path = globalThis.__fds && globalThis.__fds[fd];
                if (path) {
                  globalThis.__rust_write_file(path, data);
                } else {
                  throw new Error("Invalid file descriptor: " + fd);
                }
              },
              closeSync(fd) {
                if (globalThis.__fds) {
                  delete globalThis.__fds[fd];
                }
              },
              mkdirSync(path, options) {
                globalThis.__rust_mkdir_sync(path, options);
              },
              utimesSync(path, atime, mtime) {
                globalThis.__rust_utimes_sync(path, atime, mtime);
              },
              unlinkSync(path) {
                globalThis.__rust_unlink_sync(path);
              },
              realpathSync,
              watchFile(fileName, options, listener) { return { close() {} }; },
              unwatchFile(fileName, listener) {},
              watch(fileName, options, listener) { return { close() {} }; }
          },
          path: {
              join(...args) { return globalThis.__rust_path_join(...args); },
              dirname(p) { return globalThis.__rust_path_dirname(p); },
              basename(p) { return globalThis.__rust_path_basename(p); },
              resolve(...args) { return globalThis.__rust_path_resolve(...args); }
          },
          os: {
              platform() { return "linux"; },
              EOL: "\n"
          }
        };

        const modulesCache = {};

        function evaluateCommonJS(resolvedPath, exports, require, module, __filename, __dirname) {
          const code = globalThis.__rust_read_file(resolvedPath);
          const wrapper = new Function("exports", "require", "module", "__filename", "__dirname", code);
          wrapper(exports, require, module, __filename, __dirname);
        }

        globalThis.exports = {};
        globalThis.module = { exports: globalThis.exports };

        globalThis.require = function require(mod) {
          if (builtinModules[mod]) {
            return builtinModules[mod];
          }

          if (mod.startsWith(".") || mod.startsWith("/")) {
            const currentDir = globalThis.__current_require_dirname || globalThis.__dirname;
            const resolvedPath = globalThis.__rust_path_resolve(
              globalThis.__rust_path_join(currentDir, mod)
            );

            if (modulesCache[resolvedPath]) {
              return modulesCache[resolvedPath].exports;
            }

            const moduleObj = { exports: {} };
            modulesCache[resolvedPath] = moduleObj;

            const nextDir = globalThis.__rust_path_dirname(resolvedPath);
            const prevDir = globalThis.__current_require_dirname;
            globalThis.__current_require_dirname = nextDir;

            try {
              evaluateCommonJS(
                resolvedPath,
                moduleObj.exports,
                require,
                moduleObj,
                resolvedPath,
                nextDir
              );
            } finally {
              globalThis.__current_require_dirname = prevDir;
            }

            return moduleObj.exports;
          }

          throw new Error("Module not found: " + mod);
        };
