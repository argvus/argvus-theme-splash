# argvus-theme-splash

GTK4 Wayland layer-shell overlay displayed while ARGVUS applies a theme.

The executable is an internal desktop component and is installed at
`/usr/lib/argvus/theme-splash/splash`; it is intentionally not placed in
`/usr/bin`.

## Development

```sh
make check
make validate
make package
```

The local package helper includes the sibling `argvus-i18n` checkout required
by the Cargo workspace.
