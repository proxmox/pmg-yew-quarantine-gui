include /usr/share/dpkg/default.mk

PACKAGE=pmg-mobile-quarantine-ui
CRATENAME=pmg-mobile-quarantine-ui

BUILDDIR ?= $(PACKAGE)-$(DEB_VERSION_UPSTREAM)
ORIG_SRC_TAR=$(PACKAGE)_$(DEB_VERSION_UPSTREAM).orig.tar.gz

DEB=$(PACKAGE)_$(DEB_VERSION)_$(DEB_HOST_ARCH).deb
DSC=$(PACKAGE)_$(DEB_VERSION).dsc

# TODO: adapt for yew ui
CARGO ?= cargo
ifeq ($(BUILD_MODE), release)
CARGO_BUILD_ARGS += --release
COMPILEDIR := target/release
else
COMPILEDIR := target/debug
endif

DESTDIR =
PREFIX = /usr
UIDIR = $(PREFIX)/share/$(PACKAGE)

COMPILED_OUTPUT := \
	dist/$(CRATENAME)_bundle.js \
	dist/$(CRATENAME)_bg.wasm.gz \
	dist/mobile-yew-style.css \

all: $(COMPILED_OUTPUT)

dist:
	mkdir dist

dist/$(CRATENAME).js dist/$(CRATENAME)_bg.wasm &: $(shell find src -name '*.rs')
	proxmox-wasm-builder build -n $(CRATENAME) --release --optimize

.PHONY: rebuild
rebuild:
	proxmox-wasm-builder build -n $(CRATENAME) --release

dist/$(CRATENAME)_bg.wasm.gz: dist/$(CRATENAME)_bg.wasm
	gzip -c9 $^ > $@

dist/$(CRATENAME)_bundle.js: dist/$(CRATENAME).js dist/$(CRATENAME)_bg.wasm
	esbuild --bundle dist/$(CRATENAME).js --format=esm > dist/$(CRATENAME)_bundle.js.tmp
	mv dist/$(CRATENAME)_bundle.js.tmp dist/$(CRATENAME)_bundle.js

dist/%.css: pwt-assets/scss/%.scss dist
	rust-grass $< $@

install: $(COMPILED_OUTPUT) pmg-mobile-index.html.tt
	install -dm0755 $(DESTDIR)$(UIDIR)
	install -dm0755 $(DESTDIR)$(UIDIR)/css

	install -dm0755 $(DESTDIR)$(UIDIR)/images
	install -m0644 images/proxmox_logo.svg $(DESTDIR)$(UIDIR)/images
	install -m0644 images/proxmox_logo_white.svg $(DESTDIR)$(UIDIR)/images

	install -dm0755 $(DESTDIR)$(UIDIR)/fonts
	install -m0644 pwt-assets/assets/fonts/RobotoFlexVariableFont.ttf $(DESTDIR)$(UIDIR)/fonts
	install -m0644 pwt-assets/assets/fonts/RobotoFlexVariableFont.woff2 $(DESTDIR)$(UIDIR)/fonts

	install -m0644 dist/$(CRATENAME)_bundle.js $(DESTDIR)$(UIDIR)/
	install -m0644 dist/$(CRATENAME)_bg.wasm.gz $(DESTDIR)$(UIDIR)/
	install -m0644 dist/mobile-yew-style.css $(DESTDIR)$(UIDIR)/css
	install -m0644 pmg-mobile-index.html.tt $(DESTDIR)$(UIDIR)


$(BUILDDIR):
	rm -rf $@ $@.tmp
	mkdir -p $@.tmp
	cp -a debian/ src/ pwt-assets/ images/ pmg-mobile-index.html.tt Makefile Cargo.toml $@.tmp
	echo "git clone git://git.proxmox.com/git/$(PACKAGE).git\\ngit checkout $$(git rev-parse HEAD)" \
	    > $@.tmp/debian/SOURCE
	mv $@.tmp $@

$(ORIG_SRC_TAR): $(BUILDDIR)
	tar czf $(ORIG_SRC_TAR) --exclude="$(BUILDDIR)/debian" $(BUILDDIR)

.PHONY: deb
deb: $(DEB)
$(DEB): $(BUILDDIR)
	cd $(BUILDDIR); dpkg-buildpackage -b -uc -us
	lintian $(DEB)
	@echo $(DEB)

.PHONY: dsc
dsc: $(BUILDDIR)
	rm -rf $(DSC) $(BUILDDIR)
	$(MAKE) $(DSC)
	lintian $(DSC)

$(DSC): $(BUILDDIR) $(ORIG_SRC_TAR)
	cd $(BUILDDIR); dpkg-buildpackage -S -us -uc -d

sbuild: $(DSC)
	sbuild $(DSC)

.PHONY: upload
upload: UPLOAD_DIST ?= $(DEB_DISTRIBUTION)
upload: $(DEB)
	tar cf - $(DEB) |ssh -X repoman@repo.proxmox.com -- upload --product pmg --dist $(UPLOAD_DIST) --arch $(DEB_HOST_ARCH)

.PHONY: clean distclean
distclean: clean
clean:
	$(CARGO) clean
	rm -rf $(PACKAGE)-[0-9]*/ build/ dist/
	rm -f *.deb *.changes *.dsc *.tar.* *.buildinfo *.build .do-cargo-build

.PHONY: dinstall
dinstall: deb
	dpkg -i $(DEB)
