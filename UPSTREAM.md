# Upstream Policy

Octoscript does not carry its own copy of the Makepad VM. The language VM,
parser, derive crate, and their leaf dependencies come from the `octoscript`
branch of the OctoSense fork of Makepad, consumed as a pinned git dependency:

```toml
makepad-script = { git = "https://github.com/OctoSense-org/makepad.git", rev = "a772b3708e722a22ee9e4717d6ede8a16587f156" }
```

The pin appears in exactly two manifests, `crates/octoscript-core/Cargo.toml`
and `crates/octoscript-capabilities/Cargo.toml`, and must be the same rev in
both. `fuzz/Cargo.toml` reaches the VM only through path dependencies on those
crates and carries no pin of its own; if it ever depends on `makepad-script`
directly it must use the same rev. `Cargo.lock` records the resolved checkout.

The `octoscript` branch descends from `makepad/makepad dev` (the original
import point was `4f9ce7a8bb3fd19e5c61dcf13edd2e6d4a04cefc`). Octoscript's VM
hardening, which this repository previously carried as local vendor patches,
was ported to that branch on 2026-09-13 in fork PR #1 and now lives there.
`docs/vm-hardening-history.md` keeps the description of those changes as a
history note. This repository no longer carries local patches to the VM: a VM
change is made as a pull request against the fork's `octoscript` branch, then
reaches Octoscript through a rev bump.

Only the VM, parser, derive crate, and their direct leaf dependencies are used.
Makepad widgets, platform scripting, filesystem, process, timer, and network
modules are not depended on. New host capabilities belong in
`crates/octoscript-capabilities`, never in the VM. A behavioral change to the
VM is allowed only when it implements a published Octoscript language
contract, carries focused compatibility and streaming regressions, and does
not add ambient host authority to the VM.

The VM is not a workspace member, so its upstream lint backlog does not dilute
the lint gate for Octoscript-owned crates. Its test suite is still run
explicitly, resolved from the pinned checkout:

```sh
cargo test -p makepad-script
cargo test -p makepad-regex
```

## Bumping the pin

1. Land the change on the fork's `octoscript` branch first.
2. Move `rev` to the new commit in both manifests and run
   `cargo update -p makepad-script` so `Cargo.lock` follows.
3. Run the Octoscript regression suite and the explicit `makepad-script` and
   `makepad-regex` tests above, especially streaming precedence and
   `try/catch` recovery.
4. Bump OctoSense and Octoscript-Makepad to the same rev in the same change
   window. The three consumers are expected to track one commit of the
   `octoscript` branch; a lone bump here is a divergence, not a release.

Syncing the fork with `makepad/makepad` upstream happens on the fork, not
here; Octoscript sees the result as an ordinary rev bump.
