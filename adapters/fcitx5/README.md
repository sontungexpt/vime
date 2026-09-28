# Vietnamese Fcitx5 Adapter

Native Fcitx5 adapter plugin for Vime. Vietnamese engine logic lives in the
standalone `vime-engine` repository (github.com/sontungexpt/vime-engine), which
is pulled in via CMake `FetchContent`; the `ffi` crate (`libvime.so`) provides
the C ABI backend (`vime_engine.h`).

The native C++ addon implements `fcitx::InputMethodEngine` and links the
Rust-compiled `libvime.so` through the C ABI.

## Build

Build and install the Fcitx5 addon:

```sh
cmake -B build -DCMAKE_INSTALL_PREFIX=/usr    # see "Where to install" below
cmake --build build                            # builds its FFI crate and links libvime
sudo cmake --install build
```

`vime-engine` is resolved automatically: in a development layout (the engine
checkout next to this repo) CMake builds against the in-tree copy; otherwise it
clones `https://github.com/sontungexpt/vime-engine.git` into `build/_deps/` on
first configure — the release/standalone path. Force a specific location with
`-DFETCHCONTENT_SOURCE_DIR_VIME_ENGINE=/path/vime/engine`.

For testing the current `engine/` work end to end, read on.

---

## Where to install

**Use `/usr`.** The addon library must land in `/usr/lib/fcitx5/`, because that is
the only directory this machine's fcitx5 searches.

This is not a convention, it is a measured property. Linking against fcitx5's
own library and asking it directly:

```cpp
// fcitx::StandardPaths::global().directories(StandardPathsType::Addon, …)
User   count=0 :  (empty)
System count=1 :  [/usr/lib/fcitx5]
```

There is **no user addon directory on this system**, so `~/.local/lib/fcitx5` is
never consulted. An addon installed there builds and installs cleanly, then
fails at load with:

```
sharedlibraryloader.cpp: Could not locate library vime.so for addon vime.
```

Both symptoms are easy to misread as a broken build, so check this first when
the addon will not load:

```sh
# should say /usr
grep CMAKE_INSTALL_PREFIX build/CMakeCache.txt
```

If it says anything else, the prefix has reverted:

```sh
cmake -B build -DCMAKE_INSTALL_PREFIX=/usr
```

**`--prefix` does not work.** The `install()` rules in `CMakeLists.txt` resolve
to absolute destinations, so the prefix is baked in at *configure* time and
`cmake --install build --prefix …` is ignored:

```console
$ cmake --install build --prefix /tmp/x
CMake Error: file INSTALL cannot copy file ".../build/vime.so" to
  "/usr/local/lib/fcitx5/vime.so": Permission denied.
```

Note the error names `/usr/local`, not your `--prefix` — that is how you can
tell the flag was dropped. Use `DESTDIR` if you want a scratch install without
touching the real prefix:

```sh
DESTDIR=/tmp/stage cmake --install build
```

Because the install is root-owned, `sudo` is required. If the addon ever
migrates to a machine where fcitx5 *does* have a user addon directory,
`~/.local` becomes usable again.

---

## Testing engine changes in a live fcitx5 session

### 1. Build

```sh
cd /home/stilux/Data/workspace/vime/adapters/fcitx5
cmake -B build -DCMAKE_INSTALL_PREFIX=/usr
cmake --build build -j4
```

`cmake -B build` resolves the engine automatically. In this layout it picks the
in-tree sibling checkout and says so:

```
-- vime-engine: using in-tree checkout ../../engine
```

Confirm it in the cache before trusting it — this is the difference between
testing your work and silently testing a GitHub clone:

```sh
grep FETCHCONTENT_SOURCE_DIR_VIME_ENGINE build/CMakeCache.txt
# FETCHCONTENT_SOURCE_DIR_VIME_ENGINE:PATH=.../adapters/fcitx5/../../engine
```

The `vime-ffi-backend` target always runs cargo, so Rust changes are picked up
without reconfiguring. It is cheap when nothing changed:

```
[ 20%] Built target vime-ffi-backend
```

### 2. Verify the artifacts before installing

```sh
# the addon must want the NEW backend name
ldd build/vime.so | grep vime
#   libvime.so => .../build/rust-target/release/libvime.so

# the backend must be freshly built
ls -la build/rust-target/release/libvime.so

# all 9 C entry points must be exported
nm -D build/rust-target/release/libvime.so | grep -c 'T vime_'
#   9
```

If the first line says `libvime_ffi.so`, you are building a stale tree.

### 3. Install and reload

```sh
sudo cmake --install build
fcitx5 -r
```

This installs four files into the only paths fcitx5 reads (see "Where to
install" above):

```
/usr/lib/fcitx5/vime.so
/usr/lib/fcitx5/libvime.so
/usr/share/fcitx5/addon/vime.conf
/usr/share/fcitx5/inputmethod/vime.conf
```

Then confirm the addon actually loaded, rather than assuming:

```sh
fcitx5 -r 2>&1 | grep -i vime
# Loaded addon vime
```

`Loaded addon vime` is the success signal. If instead you see
`Could not locate library vime.so`, the prefix is wrong — go back to step 1.

The addon is `OnDemand=True`, so fcitx5 loads it per input context when a
window actually uses the method. To see which library a running fcitx5 has
mapped:

```sh
P=$(pgrep -x fcitx5 | head -1); ls -la /proc/$P/map_files/ | grep -i vime
```

Use a variable for the pid rather than an inline `$(...)`: if `pgrep` returns
empty the inline form silently becomes `/proc//map_files/`, and the error reads
like a permissions problem instead of a missing process.

### 4. Test the engine alone, before involving fcitx5

Much faster to iterate on than restarting fcitx5:

```sh
cd /home/stilux/Data/workspace/vime/engine
cargo test --workspace
```

The soundness tests for the `InlineVec` work need Miri — `cargo test` alone will
not catch the dangling-iterator class of bug:

```sh
rustup +nightly component add miri        # once
cd core
cargo +nightly miri test --test inline_vec
```

### 5. Enable and verify

Add to `~/.config/fcitx5/profile`:

```ini
[Groups/0]
Default Layout=us
DefaultIM=keyboard-us

[Groups/0/Items/0]
Name=keyboard-us
Layout=

[Groups/0/Items/1]
Name=vi
Layout=
```

The input method is registered under the name in `vime.conf`, which is `vime`:

```ini
[Groups/0/Items/1]
Name=vime
Layout=
```

Then check the active method with `fcitx5-remote -n`; it should print `vime`.

### 6. Rebuilding after engine changes

`engine/` is a **separate git repository**, so a `git checkout` inside it
silently changes what you are testing. Nothing needs committing to build — the
CMake path points at the working tree — but be aware of it.

```sh
cd /home/stilux/Data/workspace/vime/adapters/fcitx5
cmake --build build -j4
sudo cmake --install build
fcitx5 -r
```

No reconfigure needed: the `vime-ffi-backend` target always runs cargo, and the
prefix is already baked in.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| `Could not locate library vime.so` | prefix is not `/usr`, so the library is somewhere fcitx5 does not search | `grep CMAKE_INSTALL_PREFIX build/CMakeCache.txt`, then `cmake -B build -DCMAKE_INSTALL_PREFIX=/usr && sudo cmake --install build` |
| `ldd vime.so` shows `libvime_ffi.so` | stale backend from before the crate rename | rebuild; check `ldd build/vime.so` before installing |
| edits have no effect | fcitx5 has the addon cached | `fcitx5 -r` |
| `vime` not in the method list | `inputmethod/vime.conf` missing | re-run `sudo cmake --install build` |
| `ls: /proc//map_files/` | `pgrep` returned empty, not a permissions problem | assign the pid to a variable first: `P=$(pgrep -x fcitx5 \| head -1)` |
| builds the GitHub clone, not your tree | FetchContent fallback | `grep FETCHCONTENT build/CMakeCache.txt` |

## Cleaning up older installs

An addon left in a directory fcitx5 *does* search will shadow the current one,
and a stray `vime.conf` in any data directory registers a second copy that
cannot be loaded. If you have installed here before, check for leftovers:

```sh
ls -la /usr/local/lib/fcitx5/vime.so /usr/local/share/fcitx5/addon/vime.conf \
      ~/.local/lib/fcitx5/vime.so    ~/.local/share/fcitx5/addon/vime.conf 2>/dev/null
```

Anything outside `/usr` is invisible to this fcitx5's library search, but its
*config* is still read, so the `~/.local` conf is worth removing to avoid a
phantom second registration:

```sh
rm -f ~/.local/lib/fcitx5/vime.so ~/.local/lib/fcitx5/libvime.so
rm -f ~/.local/share/fcitx5/addon/vime.conf ~/.local/share/fcitx5/inputmethod/vime.conf
```

## Skipping the install while iterating

`build/vime.so` uses `INSTALL_RPATH=$ORIGIN`, so it needs `libvime.so` beside
it. Staging a runnable pair elsewhere is useful for running the in-tree build
under a test harness:

```sh
mkdir -p /tmp/vime-run
cp build/vime.so build/rust-target/release/libvime.so /tmp/vime-run/
```

Trivially reversible with `rm -rf /tmp/vime-run`. This does **not** replace the
system install: fcitx5 will not load an addon from an arbitrary directory, so
this is for linking a test binary against the fresh build, not for testing
through fcitx5 itself.