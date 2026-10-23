# Third-party software

Sprixel is released under the [MIT License](../LICENSE). It uses the open
source software below. This page lists what needs a notice and the result of
the license check made for the demo.

## Fonts (SIL Open Font License 1.1)

The app bundles three fonts from the `@fontsource` packages. They are free to
use, bundle and redistribute under the
[SIL Open Font License 1.1](https://openfontlicense.org/open-font-license-official-text/).
The copyright notices must travel with the fonts.

| Font | Used for | Copyright |
| --- | --- | --- |
| Space Grotesk | Titles | Copyright 2020 The Space Grotesk Project Authors (https://github.com/floriankarsten/space-grotesk) |
| Inter | Interface text | Copyright 2016 The Inter Project Authors (https://github.com/rsms/inter) |
| DM Mono | Keys, numbers and code | Copyright 2020 The DM Mono Project Authors (https://www.github.com/googlefonts/dm-mono) |

The font names are reserved by their authors. Sprixel does not change the
fonts.

> Release builds must ship this file (or the full licenses of the fonts) with
> the app. This is part of the `install-and-release` spec.

## License check

The check was made on 2026-10-23 for the `dev` branch.

- **JavaScript, production dependencies:** MIT, Apache-2.0 or MIT, and the
  three OFL-1.1 fonts above.
- **JavaScript, build tools:** the build tool `lightningcss` (used by Tailwind)
  is MPL-2.0. It runs only while building and is not part of the app.
- **Rust (432 packages, including Tauri):** almost all MIT or Apache-2.0, plus
  Unicode-3.0, ISC, BSD-3-Clause, Zlib and Unlicense. No GPL or AGPL.
  - Five crates are MPL-2.0: `cssparser`, `cssparser-macros`, `dtoa-short`,
    `option-ext` and `selectors`. MPL-2.0 only applies to the files of those
    crates, which Sprixel does not change. Their source is available at the
    links in their `Cargo.toml`.
  - `r-efi` is offered under `MIT OR Apache-2.0 OR LGPL-2.1-or-later`; Sprixel
    uses it under MIT or Apache-2.0.

Check it again before a release, because the dependencies change:

```sh
npx license-checker --production --summary
cargo metadata --format-version 1 --locked --manifest-path src-tauri/Cargo.toml
```

## Design assets

The icon, the logo and the screens in `docs/design/` were made for Sprixel and
are covered by the same MIT License.
