# Maintainer: Mahmud Ridwan <m@hjr265.me>
pkgname=utools
pkgver=0.1.0
pkgrel=1
epoch=
pkgdesc="A small utility suite written in Rust"
arch=('x86_64')
url="https://github.com/hjr265/utools"
license=('MIT')
depends=()
makedepends=('rustup' 'cargo' 'clang' 'tree-sitter')
source=()
sha256sums=()

build() {
    cd "$srcdir/.."

    # Assume built
    # cargo build --release --frozen
}

package() {
    cd "$srcdir/.."

    # Binary
    install -Dm755 "target/release/utools" \
        "$pkgdir/usr/bin/utools"

    # Desktop Entry
    install -Dm644 "assets/desktop/utools.desktop" \
        "$pkgdir/usr/share/applications/utools.desktop"

    # Icon
    # install -Dm644 "assets/icons/utools.png" \
    #     "$pkgdir/usr/share/icons/hicolor/256x256/apps/utools.png"
}
