# Release 0.1.43

- [Disable `rustc-serialize` derives for future compilers.][132]

[132]: https://github.com/rust-num/num-rational/pull/132

# Release 0.1.42

- Maintenance release to update dependencies.


# Release 0.1.41

- [num-rational now has its own source repository][num-356] at [rust-num/num-rational][home].
- [`Ratio` now implements `CheckedAdd`, `CheckedSub`, `CheckedMul`, and `CheckedDiv`][11].
- [`Ratio` now implements `AddAssign`, `SubAssign`, `MulAssign`, `DivAssign`, and `RemAssign`][12]
  with either `Ratio` or an integer on the right side.  The non-assignment operators now also
  accept integers as an operand.
- [`Ratio` operators now make fewer `clone()` calls][14].

Thanks to @c410-f3r, @cuviper, and @psimonyi for their contributions!

[home]: https://github.com/rust-num/num-rational
[num-356]: https://github.com/rust-num/num/pull/356
[11]: https://github.com/rust-num/num-rational/pull/11
[12]: https://github.com/rust-num/num-rational/pull/12
[14]: https://github.com/rust-num/num-rational/pull/14


# Prior releases

No prior release notes were kept.  Thanks all the same to the many
contributors that have made this crate what it is!
