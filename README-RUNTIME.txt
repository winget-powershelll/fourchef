4chef Runtime Bundle

Contents:
- bin/fourchef                     : standalone release binary
- run-fourchef.sh                  : launcher script
- packages/4chef_0.1.0_amd64.deb  : Debian/Ubuntu installer package
- packages/4chef-0.1.0-1.x86_64.rpm: RPM installer package
- packages/4chef_0.1.0_amd64.AppImage: portable AppImage package
- LICENSE

Run directly (portable style):
  ./run-fourchef.sh

Install package instead (system install):
  Debian/Ubuntu: sudo apt install ./packages/4chef_0.1.0_amd64.deb
  Arch/Fedora/openSUSE (rpm tools): sudo rpm -i ./packages/4chef-0.1.0-1.x86_64.rpm
  Portable AppImage: chmod +x ./packages/4chef_0.1.0_amd64.AppImage && ./packages/4chef_0.1.0_amd64.AppImage

Notes:
- This bundle intentionally excludes export/source data files.
- The app still writes its working SQLite DB under the user's app-data directory.
