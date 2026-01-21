# Safety Notes (FFI / Threading)

This crate is a safe Rust wrapper around the PicoGK native library.
Because the implementation crosses an FFI boundary, there are a few important
assumptions and contracts that must hold for safety.

## 1) Native Handle Types

Most core types (`Voxels`, `Mesh`, `Lattice`, `ScalarField`, `VectorField`, `VdbFile`, `Viewer`)
internally store an opaque native handle (raw pointer).

All public methods validate obvious null-handle cases at construction time, but the native
library ultimately owns the memory and invariants.

## 2) Threading Model / Send + Sync

The wrapper marks several handle types as `Send`/`Sync` via `unsafe impl`.
This is made sound by a **process-global, re-entrant FFI lock** (`src/ffi_lock.rs`) that
serializes all calls into the native PicoGK library.

Implications:
- Concurrent PicoGK calls from multiple threads are allowed but **executed one-at-a-time**.
- Re-entrant calls from callbacks on the *same* thread are supported by the lock.

## 3) Callback Bridging (Implicit / TraverseActive)

Some APIs bridge Rust callbacks into native code:

- `Voxels::render_implicit` / `Voxels::intersect_implicit`
- `ScalarField::traverse_active`
- `VectorField::traverse_active`

These are implemented using a process-global pointer + C trampoline.

Contract:
- The native library must invoke the callback **synchronously** during the FFI call.
- The callback bridge is **not re-entrant** (nested / concurrent traversals are rejected).

If the native side ever calls the callback asynchronously (after the FFI function returns),
that would be undefined behavior for any wrapper and must be treated as unsupported.

## 4) Viewer Main-Thread Rule

`Viewer::poll()` must be called from the same thread that created the viewer.
Calling it from a different thread logs an error and returns `false`.

## 5) Viewer Object Lifetimes (Handle-Only APIs)

Some viewer APIs only pass the native handle across the action queue.
Because actions are processed asynchronously during `Viewer::poll()`, borrowing an object and
only storing its native handle can lead to use-after-free if the object is dropped before the
action is applied.

The following *handle-only add* APIs are therefore `unsafe`:

- `unsafe Viewer::add_mesh(&Mesh, ...)`
- `unsafe Viewer::add_polyline_ref(&PolyLine, ...)`

Safety contract:
- The viewer does not take ownership of the object.
- The caller must keep the object alive until it is removed from the viewer **and** the remove
  action has been processed (i.e. after a subsequent `poll()`).

`Viewer::add_voxels(&Voxels, ...)` is safe because voxels are converted into a mesh eagerly at
enqueue time; the viewer does not keep or later dereference the voxels handle.

If you want the viewer to keep objects alive, prefer the owned/shared variants:

- `Viewer::add_voxels_owned(Voxels, ...)`
- `Viewer::add_voxels_shared(Arc<Voxels>, ...)`
- `Viewer::add_mesh_owned(Mesh, ...)`
- `Viewer::add_mesh_shared(Arc<Mesh>, ...)`
- `Viewer::add_polyline(PolyLine, ...)`
- `Viewer::add_polyline_shared(Arc<PolyLine>, ...)`

Or use the C#-style convenience wrappers:

- `Viewer::add(...)` returns an `Arc<...>` handle (for later removal) and always uses an owned/shared path internally
- `Viewer::remove(&Arc<...>)` keeps the object alive until the remove action is processed
