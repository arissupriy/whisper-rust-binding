# Manual Fix Steps for Rust Code Issues

If the automatic script doesn't work, follow these manual steps to fix the Rust code issues:

## 1. Fix FFI module declaration

In `src/lib.rs`, find the line:

```rust
mod ffi {
    use super::*;

    unsafe unsafe extern "C" {
```

Change it to:

```rust
mod ffi {
    use super::*;

    unsafe extern "C" {
```

## 2. Fix exported C functions

For each `#[no_mangle]` function, add the `unsafe` keyword.

Find this pattern:

```rust
#[no_mangle]
pub extern "C" fn function_name(...) {
```

Change to:

```rust
#[no_mangle]
pub unsafe extern "C" fn function_name(...) {
```

## 3. Fix FFI function calls

Wrap all calls to FFI functions in `unsafe` blocks:

```rust
// Change this:
let success = whisper_rust_process_audio(
    // parameters...
);

// To this:
let success = unsafe { whisper_rust_process_audio(
    // parameters...
) };
```

Do this for all calls to:
- `whisper_rust_process_audio`
- `whisper_rust_process_audio_sliding_window`
- `whisper_rust_get_model_info`
- `whisper_rust_validate_word`

## 4. Fix duplicate unsafe blocks

If you find patterns like:

```rust
unsafe { unsafe { whisper_rust_validate_word(
```

Change to:

```rust
unsafe { whisper_rust_validate_word(
```

## 5. Build and test

After making these changes, try building:

```bash
cargo build --release
```

If it succeeds, your library will be available at `target/release/libwhisper_rust.so`.
