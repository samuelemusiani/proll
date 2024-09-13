# Proll

Make pacman package rollbacks easy with a simple rust program

# Usage

To list all packages that match a patter use the `list` command:

```bash
$ proll list <PACKAGE>
$ proll list cadd
Name		Version		Build	Arch
caddy		2.7.5		1	x86_64
caddy		2.7.6		1	x86_64
caddy		2.8.4		1	x86_64
```

To downgrade a package to a specific version use the `downgrade` command:
```bash
$ proll downgrade <PACKAGE> [VERSION]
$ proll downgrade caddy 2.7.6
$ proll downgrade caddy-2.7.6-1-x86_64
```
