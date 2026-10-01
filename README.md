# argvus-theme-splash

GTK4 Wayland layer-shell overlay displayed while ARGVUS applies a theme or
starts the desktop session.

The executable is an internal desktop component and is installed at
`/usr/lib/argvus/theme-splash/splash`; it is intentionally not placed in
`/usr/bin`.

The session loading path invokes the same binary with `--spinner-only`. In
that mode the overlay reads the selected theme from
`$ARGVUS_CONFIG_HOME/argvus/data/.active-theme` (falling back to the packaged ARGVUS
Dark palette), displays only the non-interactive spinner, and exits on `SIGTERM`.
Readiness is reported only from the first GTK frame callback of every mapped
layer-shell surface; it is not inferred from process startup or `present()`.

## Development

```sh
make check
make validate
make package
```

The local package helper includes the sibling `argvus-i18n` checkout required
by the Cargo workspace.
