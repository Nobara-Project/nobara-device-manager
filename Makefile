all:
	true

build:
	cargo fetch
	cargo build --release

build_debug:
	cargo fetch
	cargo build

install_no_build:
	mkdir -p $(DESTDIR)/usr/bin/
	cp -vf target/release/nobara-device-manager $(DESTDIR)/usr/bin/
	chmod 755 $(DESTDIR)/usr/bin/nobara-device-manager
	mkdir -p $(DESTDIR)/usr/share/glib-2.0/schemas/
	cp data/*.xml $(DESTDIR)/usr/share/glib-2.0/schemas/
	mkdir -p $(DESTDIR)/usr/share/applications
	mkdir -p $(DESTDIR)/usr/share/icons/hicolor/scalable/apps
	cp -vf data/com.github.nobara-project.nobaradevicemanager.svg $(DESTDIR)/usr/share/icons/hicolor/scalable/apps/
	cp -vf data/com.github.nobara-project.nobaradevicemanager.desktop  $(DESTDIR)/usr/share/applications/

install_no_build_debug:
	mkdir -p $(DESTDIR)/usr/bin/
	cp -vf target/debug/nobara-device-manager $(DESTDIR)/usr/bin/
	chmod 755 $(DESTDIR)/usr/bin/nobara-device-manager
	mkdir -p $(DESTDIR)/usr/share/glib-2.0/schemas/
	cp data/*.xml $(DESTDIR)/usr/share/glib-2.0/schemas/
	mkdir -p $(DESTDIR)/usr/share/applications
	mkdir -p $(DESTDIR)/usr/share/icons/hicolor/scalable/apps
	cp -vf data/com.github.nobara-project.nobaradevicemanager.svg $(DESTDIR)/usr/share/icons/hicolor/scalable/apps/
	cp -vf data/com.github.nobara-project.nobaradevicemanager.desktop  $(DESTDIR)/usr/share/applications/

install:
	mkdir -p $(DESTDIR)/usr/bin/
	cargo fetch
	cargo build --release
	cp -vf target/release/nobara-device-manager $(DESTDIR)/usr/bin/
	chmod 755 $(DESTDIR)/usr/bin/nobara-device-manager
	mkdir -p $(DESTDIR)/usr/share/glib-2.0/schemas/
	cp data/*.xml $(DESTDIR)/usr/share/glib-2.0/schemas/
	mkdir -p $(DESTDIR)/usr/share/applications
	mkdir -p $(DESTDIR)/usr/share/icons/hicolor/scalable/apps
	cp -vf data/com.github.nobara-project.nobaradevicemanager.svg $(DESTDIR)/usr/share/icons/hicolor/scalable/apps/
	cp -vf data/com.github.nobara-project.nobaradevicemanager.desktop  $(DESTDIR)/usr/share/applications/